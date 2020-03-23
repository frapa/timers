use std::collections::HashMap;

use colored::*;
use itertools::{Itertools, enumerate};
use crate::util::parse_int;

trait TaskPrinter {
    fn print_header(&self);
    fn print_ellipsis(&self);
    fn print_task(&self, task: &timers::Task);

    fn print_tasks(&self, tasks: HashMap<u32, timers::Task>, num: usize) {
        self.print_header();

        if tasks.len() > num {
            self.print_ellipsis();
        }

        for (i, id) in enumerate(tasks.keys().sorted()) {
            if tasks.len() > num && i < tasks.len()-num {
                continue
            }

            let task = &tasks[id];
            self.print_task(task);
        }
    }
}

struct ShortPrinter ();

impl TaskPrinter for ShortPrinter {
    fn print_header(&self) {
        println!("{:<6} {:<36} {}", "ID", "TASK", "DURATION");
    }

    fn print_ellipsis(&self) {
        println!("{:<6} {:<36} {}", "...", "...", "...");
    }

    fn print_task(&self, task: &timers::Task) {
        match task.status() {
            timers::TaskStatus::Logging() => println!(
                "{:<6} {:<36} {}",
                format!("@{}", task.id).yellow().bold(),
                task.name.red().bold(),
                timers::format_duration(task.duration()).bold(),
            ),
            timers::TaskStatus::Stopped() => println!(
                "{:<6} {:<36} {}",
                format!("@{}", task.id),
                task.name,
                timers::format_duration(task.duration()),
            ),
        }
    }
}

struct LongPrinter ();

impl TaskPrinter for LongPrinter {
    fn print_header(&self) {
        println!(
            "{:<6} {:<36} {:<14} {:<8} {:<6} {}",
             "ID", "TASK", "DURATION", "STATUS", "LOGS", "LAST LOG"
        );
    }

    fn print_ellipsis(&self) {
        println!(
            "{:<6} {:<36} {:<14} {:<8} {:<6} {}",
            "...", "...", "...", "...", "...", "..."
        );
    }

    fn print_task(&self, task: &timers::Task) {
        let last = task.logs.last().unwrap()
            .start.with_timezone(&chrono::Local)
            .format("%a %b %d %H:%M").to_string();

        match task.status() {
            timers::TaskStatus::Logging() => println!(
                "{:<6} {:<36} {:<14} {:<8} {:<6} {}",
                format!("@{}", task.id).yellow().bold(),
                task.name.red().bold(),
                timers::format_duration(task.duration()).bold(),
                task.status_text().bold(),
                format!("{}", task.logs.len()).bold(),
                last.bold(),
            ),
            timers::TaskStatus::Stopped() => println!(
                "{:<6} {:<36} {:<14} {:<8} {:<6} {}",
                format!("@{}", task.id),
                task.name,
                timers::format_duration(task.duration()),
                task.status_text(),
                task.logs.len(),
                last,
            ),
        }
    }
}

pub fn tasks_command(matches: &clap::ArgMatches) {
    let raw_num = matches.value_of("num").unwrap();
    let num = parse_int(raw_num).unwrap_or_else(
        |_| {
            println!("Invalid number of tasks: '{}'", raw_num);
            std::process::exit(1);
        }
    ) as usize;

    match timers::get_all_tasks() {
        Ok(tasks) => match matches.is_present("long") {
            true => LongPrinter{}.print_tasks(tasks, num),
            false => ShortPrinter{}.print_tasks(tasks, num),
        },
        Err(err) => println!("Error retrieving tasks: {}", err)
    }
}