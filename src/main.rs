extern crate libc;
extern crate getopts;

use std::ffi::CString;
use std::env;
use std::ptr;
use std::mem;
use std::thread;

pub use libc::consts::os::posix88::{SIGTERM,SIGINT};
use libc::funcs::posix88::unistd::waitpid;
use libc::funcs::posix88::unistd::fork;
use libc::funcs::posix88::unistd::execvp;
use libc::types::os::arch::posix88::pid_t;
use getopts::Options;

// signal handling {
#[macro_use]
extern crate bitflags;

bitflags!(
  #[repr(C)]
  flags SockFlag: libc::c_ulong {
    const SA_NOCLDSTOP = 0x00000001,
    const SA_NOCLDWAIT = 0x00000002,
    const SA_NODEFER   = 0x40000000,
    const SA_ONSTACK   = 0x08000000,
    const SA_RESETHAND = 0x80000000,
    const SA_RESTART   = 0x10000000,
    const SA_SIGINFO   = 0x00000004,
  }
);


#[repr(C)]
#[cfg(target_pointer_width = "32")]
#[derive(Clone, Copy)]
pub struct sigset_t {
  __val: [libc::c_ulong; 32],
}

#[repr(C)]
#[cfg(target_pointer_width = "64")]
#[derive(Clone, Copy)]
pub struct sigset_t {
  __val: [libc::c_ulong; 16],
}

#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct sigaction {
  pub sa_handler: extern fn(libc::c_int),
  pub sa_mask: sigset_t,
  pub sa_flags: SockFlag,
  sa_restorer: *mut libc::c_void,
}

extern {
  pub fn sigaction(
    signum: libc::c_int,
    act: *const sigaction,
    oldact: *mut sigaction
  ) -> libc::c_int;
}
// }

#[test]
fn it_works() {
}

static mut RUNNING: bool = true;
const WNOHANG: libc::c_int = 1;
// how often to check if the process should halt in milliseconds
const HALT_RESOLUTION: u32 = 100;

fn wait_for_commands_to_exit(pids: &mut Vec<pid_t>) {
  loop {
    let status: i32 = 0;
    let waited_pid: pid_t;
    unsafe {
      waited_pid = waitpid(-1, &status, WNOHANG);
    }
    if waited_pid == 0 {
      thread::sleep_ms(HALT_RESOLUTION);
    }
    else {
      match pids.iter().position(|p| *p == waited_pid) {
        Some(idx) => {
          pids.remove(idx);
          println!("exited {}", waited_pid);
          if pids.is_empty() {
            return;
          }
        }
        None => {}
      }
    }

    unsafe {
      if ! RUNNING {
        println!("terminated - TODO: send SIGTERM to children");
        return;
      }
    }
  }
}

fn print_usage(program: &str, opts: Options) {
  let brief = format!("usage: {} [options] program [arguments]", program);
  print!("{}", opts.usage(&brief));
}

fn run_commands(cmd_and_args: &Vec<String>) -> Vec<pid_t> {
  println!("running `{}'", cmd_and_args.connect(" "));

  let mut ret = Vec::<pid_t>::new();

  let cmds = cmd_and_args.split(|s| s == "---");
  for cmd in cmds {
    let pid;
    unsafe {
      pid = fork();
    }

    if pid == 0 {
      let mut cstrings = Vec::<CString>::new();
      let mut arg_ptrs = Vec::<*const i8>::new();

      cstrings.reserve(cmd.len());
      arg_ptrs.reserve(cmd.len() + 1);

      for arg in cmd.iter() {
        cstrings.push(CString::new(arg.clone()).unwrap());
        arg_ptrs.push(cstrings.last().unwrap().as_ptr());
      }
      arg_ptrs.push(ptr::null());

      unsafe {
        execvp(
          CString::new(cmd[0].clone()).unwrap().as_ptr(),
          arg_ptrs.as_mut_ptr()
        );
      }
      panic!("execvp failed");
    }
    else {
      ret.push(pid);
    }

  }

  return ret;
}

extern fn accept_term(_signal: libc::c_int) {
  unsafe {
    RUNNING = false;
  }
}

fn main() {
  let args: Vec<String> = env::args().collect();
  let program = args[0].clone();
  let mut opts = Options::new();
  opts.optflag("h", "help", "show help");
  let matches = match opts.parse(&args[1..]) {
    Ok(m) => { m }
    Err(f) => { panic!(f.to_string()) }
  };

  if matches.opt_present("h") {
    print_usage(&program, opts);
    return;
  }

  if matches.free.len() == 0 {
    println!("must specify a command to run");
    print_usage(&program, opts);
    return;
  }

  let mut pids: Vec<pid_t>;
  let mut sa = unsafe { mem::zeroed::<sigaction>() };
  sa.sa_handler = accept_term;

  unsafe {
    sigaction(SIGTERM, &sa, ptr::null_mut());
    sigaction(SIGINT, &sa, ptr::null_mut());
  }

  pids = run_commands(&matches.free);
  println!("pids are {:?}", pids);
  wait_for_commands_to_exit(&mut pids);
}
