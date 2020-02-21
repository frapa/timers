# `timers` - Command line time tracking tool

`timers` is a simple and effective time tracking tool with a easy-to-use
command line interface. With `timers` you can:

- Log time on tasks

`timers` is written in rust and uses simple text files to save the tasks, which make it
extremely fast and lightweight.

## How to use

To start logging time on a task, use

```bash
$ timers log "Writing timers readme"
@1: Writing timers readme
status: logging
time: 9m 4s
```

The `@1` is the task id. You can check the current logging status with
`timers status`. You can stop logging with

```bash
$ timers stop
@1: Writing timers readme
status: stopped
time: 19m 41s
```

You can then resume logging on a previous task by id:

```bash
$ timers log @1
@1: Writing timers readme
status: logging
time: 19m 41s
```

## FAQ

**A command line tool? Come-on, it's 2020!**

Command line can be very simple and let me focus my energy on more important features.
Plus it's much more efficient (battery, memory, CPU) than any GUI tool,
or even worse web-based solution.

You're free to use your web-based tool that will eat 100 MB of your memory, download 10 Mb
of crap (including the very necessary 8 Mb of javascript code) just so you can click
the "log" button on a task.