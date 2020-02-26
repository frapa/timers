use std::ops::{Sub, Add};

use clap;
use colored::*;
use itertools::Itertools;
use chrono::{Timelike, Datelike};

mod util;
use util::*;
mod basic_op;
use basic_op::*;

use timers;

fn main() {
    let matches = parse_args();

    match matches.subcommand_name() {
        Some("log") => log_command(matches.subcommand_matches("log").unwrap()),
        Some("status") => status_command(),
        Some("stop") => stop_command(),
        Some("report") => match matches.subcommand_matches("report").unwrap().subcommand_name() {
            Some("days") => report_days_command(),
            _ => report_days_command(),
        }
        Some("tasks") => tasks_command(),
        _ => {},
    }
}

fn parse_args() -> clap::ArgMatches<'static> {
    return clap::App::new("timers")
        .author("Francesco Pasa <francescopasa@gmail.com>")
        .version("0.1.0")
        .about("Track time spent on tasks")
        .setting(clap::AppSettings::ArgRequiredElseHelp)
        .subcommand(clap::SubCommand::with_name("log")
            .alias("start")
            .about("Log time on a task")
            .arg(clap::Arg::with_name("TASK")
                .required(true)
                .index(1)
                .help(
                    "Name of the task to log, or ID of an existing task, \
                        to continue logging on an existing task."
                )
            )
            .arg(clap::Arg::with_name("AT")
                .long("at")
                .takes_value(true)
                .value_name("TIME")
                .help("Start logging at the specified time.")
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
            .subcommand(clap::SubCommand::with_name("days")
                .about("Report statistics on days")
            )
        )
        .subcommand(clap::SubCommand::with_name("tasks")
            .about("Print tasks")
        )
        .get_matches();
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

fn report_days_command() {
    let week_offset = chrono::Local::now().weekday().num_days_from_monday() as i64;
    let week_start = chrono::Local::now()
        .with_hour(0).unwrap()
        .with_minute(0).unwrap()
        .with_second(0).unwrap()
        .with_nanosecond(0).unwrap()
        .sub(chrono::Duration::days(week_offset))
        .with_timezone(&chrono::Utc);

    for i in 0..7 {
        let start = week_start.add(chrono::Duration::days(i));
        let end = week_start.add(chrono::Duration::days(i+1));

        let local_start = start.with_timezone(&chrono::Local);

        match timers::get_total_duration(start, end) {
            Ok(duration) => println!(
                "{}: {}",
                if i < 5 {
                    local_start.format("%a").to_string().green()
                } else {
                    local_start.format("%a").to_string().red()
                },
                timers::format_duration(duration)
            ),
            Err(err) => println!("Error computing duration: {}", err)
        }
    }
}