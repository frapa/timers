use std::collections::HashMap;

use colored::*;
use itertools::Itertools;

pub fn tasks_command(matches: &clap::ArgMatches) {
    match timers::get_all_tasks() {
        Ok(tasks) => {
            match matches.is_present("long") {
                true => print_tasks_long(tasks),
                false => print_tasks(tasks),
            }
        },
        Err(err) => println!("Error retrieving tasks: {}", err)
    }
}

fn print_tasks(tasks: HashMap<u32, timers::Task>) {
    for id in tasks.keys().sorted() {
        let task = &tasks[id];
        match task.status() {
            timers::TaskStatus::Logging() => println!(
                "{} {} [{}]",
                format!("@{}:", task.id).yellow().bold(),
                task.name.red().bold(),
                task.status_text(),
            ),
            timers::TaskStatus::Stopped() => println!(
                "{} {} [{}]",
                format!("@{}:", task.id),
                task.name,
                task.status_text(),
            ),
        }
    }
}

fn print_tasks_long(tasks: HashMap<u32, timers::Task>) {
    for id in tasks.keys().sorted() {
        let task = &tasks[id];

        match task.status() {
            timers::TaskStatus::Logging() => println!(
                "{} {}",
                format!("@{}:", task.id).yellow().bold(),
                task.name.red().bold(),
            ),
            timers::TaskStatus::Stopped() => println!(
                "{} {}",
                format!("@{}:", task.id),
                task.name,
            ),
        }

        println!("  status: {}", task.status_text());
        println!("  duration: {}", timers::format_duration(task.duration()));
        println!(
            "  last log: {}",
            task.logs.last().unwrap()
                .start.with_timezone(&chrono::Local)
                .format("%a %b %d %H:%M")
        );
    }
}