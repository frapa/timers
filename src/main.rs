use std::io::prelude::*;

use clap;
use colored::*;
use itertools::Itertools;

use timers;

fn main() {
    let matches = parse_args();

    match matches.subcommand_name() {
        Some("log") => log_command(matches.subcommand_matches("log").unwrap()),
        Some("status") => status_command(),
        Some("stop") => stop_command(),
        Some("report") => println!("report"),
        Some("tasks") => tasks_command(),
        _ => {}
    }
}

fn parse_args() -> clap::ArgMatches<'static> {
    return clap::App::new("timers")
        .author("Francesco Pasa <francescopasa@gmail.com>")
        .version("0.1.0")
        .about("Track time spent on tasks")
        .subcommand(clap::SubCommand::with_name("log")
            .about("Log time on a task")
            .arg(clap::Arg::with_name("TASK")
                .required(true)
                .index(1)
                .help(
                    "Name of the task to log, or ID of an existing task, \
                        to continue logging on an existing task."
                )
            )
        )
        .subcommand(clap::SubCommand::with_name("status")
            .about("Get logging status")
        )
        .subcommand(clap::SubCommand::with_name("stop")
            .about("Stop logging time on the current task")
        )
        .subcommand(clap::SubCommand::with_name("report")
            .about("Report statistics on the tasks")
        )
        .subcommand(clap::SubCommand::with_name("tasks")
            .about("Print tasks")
        )
        .get_matches();
}

fn print_status(task: &timers::Task) {
    println!(
        "{} {}\nstatus: {}\ntime: {}",
        format!("@{}:", task.id).yellow().bold(),
        task.name.red().bold(),
        task.status_text().bold(),
        timers::format_duration(task.duration()).bold()
    );
//    println!(
//        "[{}] @{}: {}\n---\nlogs: {}\ntotal: {}",
//        task.status_text(),
//        task.id,
//        task.name,
//        task.logs.len(),
//        timers::format_duration(task.duration())
//    )
}

fn log_command(matches: &clap::ArgMatches) {
    // Cannot panic as the argument parser already ensures it exist
    let task = matches.value_of("TASK").unwrap();

    if task.len() == 0 {
        println!("Cannot create empty task.");
    }

    match timers::get_current_log_task() {
        Ok(Some(task)) => {
            println!(
                "Currently logging on task {} {}",
                format!("@{}:", task.id).yellow().bold(),
                task.name.red().bold(),
            );
            print!("Do you want to start the new task? [y/n] ");
            std::io::stdout().flush().unwrap();

            let mut answer = String::new();
            std::io::stdin().read_line(&mut answer).unwrap();

            if answer.trim() == "n" || answer.trim() == "no" {
                println!("aborting");
                return
            } else {
                match timers::stop_current_task() {
                    Err(err) => println!("Error finding current task: {}", err),
                    _ => {},
                }
            }
        },
        Err(err) => println!("Error finding current task: {}", err),
        _ => {},
    }

    if task.starts_with('@') {
        // This will strip multiple @ if present, currently it's not worth to fix this behavior
        let task_id_result = task.trim_start_matches('@').parse::<u32>();

        match task_id_result {
            Ok(task_id) => match timers::log_task(task_id) {
                Ok(task) => print_status(&task),
                Err(err) => println!("Error logging on task: {}", err),
            },
            Err(_) => println!("'{}' is an invalid task ID", task),
        };
    } else {
        match timers::create_log_task(task) {
            Ok(task) => print_status(&task),
            Err(err) => println!("Error creating task: {}", err),
        }
    }
}

fn status_command() {
    match timers::get_current_log_task() {
        Ok(task) => match task {
            Some(task) => print_status(&task),
            None => println!("You are not logging on any task.")
        },
        Err(err) => println!("Error finding current task: {}", err),
    }
}

fn stop_command() {
    match timers::stop_current_task() {
        Ok(task) => print_status(&task),
        Err(timers::Error::Value(_)) => println!(
            "Cannot stop because you're not logging on any task."
        ),
        Err(err) => println!("An stopping task: {}", err)
    }
}

fn tasks_command() {
    match timers::get_all_tasks() {
        Ok(tasks) => {
            for id in tasks.keys().sorted() {
                let task = &tasks[id];
                match task.status() {
                    timers::TaskStatus::Logging() => println!(
                        "{} {} [{}]",
                        format!("@{}:", task.id).yellow().bold(),
                        task.name.red().bold(),
                        task.status_text()
                    ),
                    timers::TaskStatus::Stopped() => println!(
                        "{} {} [{}]",
                        format!("@{}:", task.id),
                        task.name,
                        task.status_text()
                    ),
                }
            }
        },
        Err(err) => println!("Error retrieving tasks: {}", err)
    }
}