# A tiny init for docker containers

Hey maybe your Docker file contains something like this:

```
CMD ["/bin/node", "app.js"]
```

Oh no [this is no good!](https://blog.phusion.nl/2015/01/20/docker-and-the-pid-1-zombie-reaping-problem/): TLDR: Your process can get shutdown in a bad way and *zombie processes* will destroy you!

Instead just use `lovely_touching` and change your `Dockerfile` to this:

```
ADD lovely_touching /bin/lovely_touching
CMD ["/bin/lovely_touching", "--", "/bin/node", "app.js" ]
```

Now you don't have to worry anymore!

`lovely_touching` is written in `rust` so you don't need anything installed in the host machine to use the binary.

## More stuff

If you want to run multiple processes you can separate them with the argument `---`:
```
ADD lovely_touching /bin/lovely_touching
CMD ["/bin/lovely_touching", "--", "/bin/runit", "---", "/bin/node", "app.js" ]
```
