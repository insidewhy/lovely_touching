extern crate libc;
extern crate getopts;

use std::ffi::CString;
use std::env;
use std::ptr;

use libc::funcs::posix88::unistd::waitpid;
use libc::funcs::posix88::unistd::fork;
use libc::funcs::posix88::unistd::execvp;
use libc::types::os::arch::posix88::pid_t;
use getopts::Options;

#[test]
fn it_works() {
}

fn waitpid_reap_other_children(pid :pid_t) {
  loop {
    unsafe {
      let status :i32 = 0;
      let waited_pid = waitpid(-1, &status, 0);
      println!("exited {}", waited_pid);
      if waited_pid == pid {
        return;
      }
    }
  }
}

fn print_usage(program: &str, opts: Options) {
  let brief = format!("usage: {} [options] program [arguments]", program);
  print!("{}", opts.usage(&brief));
}

unsafe fn run_command(cmd_and_args :Vec<String>) -> pid_t {
  println!("running `{}'", cmd_and_args.connect(" "));
  let pid = fork();
  if pid == 0 {
    let mut cstrings = Vec::<CString>::new();
    let mut arg_ptrs = Vec::<*const i8>::new();

    cstrings.reserve(cmd_and_args.len());
    arg_ptrs.reserve(cmd_and_args.len() + 1);

    for arg in cmd_and_args.iter() {
      cstrings.push(CString::new(arg.clone()).unwrap());
      arg_ptrs.push(cstrings.last().unwrap().as_ptr());
    }
    arg_ptrs.push(ptr::null());

    execvp(
      CString::new(cmd_and_args[0].clone()).unwrap().as_ptr(),
      arg_ptrs.as_ptr() as (*mut *const i8)
    );
    panic!("execvp failed");
  }
  return pid;
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

  unsafe {
    let main_pid = run_command(matches.free);
    println!("pid is {}", main_pid);
    waitpid_reap_other_children(main_pid);
  }
}
