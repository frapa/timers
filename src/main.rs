extern crate clap;
extern crate rusqlite;

use clap::{App, SubCommand, Arg, ArgMatches};
use rusqlite::{Connection, Result};

fn main() {
    let matches = parse_args();

    match matches.subcommand_name() {
        Some("log") => log(matches.subcommand_matches("log").unwrap()),
        Some("stop") => println!("stop"),
        Some("report") => println!("report"),
        Some("tasks") => println!("tasks"),
        _ => {}
    }
}

fn parse_args() -> ArgMatches<'static> {
    return App::new("timers")
        .author("Francesco Pasa <francescopasa@gmail.com>")
        .version("0.1.0")
        .about("Track time spent on tasks")
        .subcommand(SubCommand::with_name("log")
            .about("Log time on a task")
            .arg(Arg::with_name("TASK")
                .required(true)
                .index(1)
                .help(
                    "Name of the task to log, or id of an existing task, \
                        to continue logging on an existing task."
                )
            )
        )
        .subcommand(SubCommand::with_name("stop")
            .about("Stop logging time on the current task")
        )
        .subcommand(SubCommand::with_name("report")
            .about("Report statistics on the tasks")
        )
        .subcommand(SubCommand::with_name("tasks")
            .about("Print tasks")
        )
        .get_matches();
}

fn log(matches: &ArgMatches) {
    // Cannot panic as the argument parser already ensures it exist
    let task = matches.value_of("TASK").unwrap();

    if task.len() == 0 {
        println!("Cannot create empty task.");
    }

    if task.starts_with('@') {
        // This will strip multiple @ if present, currently it's not worth to fix this behavior
        let task_id = task.trim_start_matches('@');
        println!("{}", task_id);
        start_log_on_task(task_id);
    } else {
        let task_id = create_task(task);
        start_log_on_task(task_id);
    }
}

fn create_task(name: &str) -> &str {
    "asd"
}

fn start_log_on_task(id: &str) {
}