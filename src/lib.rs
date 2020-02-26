use std::collections::HashMap;
use std::ops::Add;

use dirs;
use chrono;

mod errors;
pub use errors::{Error, ValueError};
mod repo;
pub use repo::{Log, Task, Repo, TaskStatus};

fn get_repo() -> Result<Repo, Error> {
    let mut path = dirs::home_dir().unwrap();
    path.push(".timers");

    // ensure folder exists
    if !path.exists() {
        std::fs::create_dir_all(&path)?;
    }

    Ok(Repo {
        path,
    })
}

pub fn create_task(name: &str) -> Result<Task, Error> {
    let repo = get_repo()?;
    Ok(repo.create_task(name)?)
}

pub fn log_task_at(id: u32, at: chrono::DateTime<chrono::Utc>) -> Result<Task, Error> {
    let repo = get_repo()?;
    let mut task = repo.get_task(id)?;
    repo.log_task(&mut task, at)?;
    Ok(task)
}

pub fn log_task(id: u32) -> Result<Task, Error> {
    Ok(log_task_at(id, chrono::Utc::now())?)
}

pub fn create_log_task_at(name: &str, at: chrono::DateTime<chrono::Utc>) -> Result<Task, Error> {
    let repo = get_repo()?;
    let mut task = repo.create_task(name)?;
    repo.log_task(&mut task, at)?;
    Ok(task)
}

pub fn create_log_task(name: &str) -> Result<Task, Error> {
    Ok(create_log_task_at(name, chrono::Utc::now())?)
}

pub fn get_current_log_task() -> Result<Option<Task>, Error> {
    let repo = get_repo()?;
    let mut tasks = repo.list_tasks()?;

    let mut found_id: Option<u32> = None;
    for (id, task) in tasks.iter() {
        if task.logging {
            found_id = Some(*id);
            break
        }
    }

    match found_id {
        Some(id) => Ok(tasks.remove(&id)),
        None => Ok(None)
    }
}

pub fn stop_current_task_at(at: chrono::DateTime<chrono::Utc>) -> Result<Task, Error> {
    let repo = get_repo()?;

    match get_current_log_task()? {
        Some(mut task) => {
            repo.stop_task(&mut task, at)?;
            Ok(task)
        },
        None => Err(Error::Value(ValueError::new("Not task currently being logged.")))
    }
}

pub fn stop_current_task() -> Result<Task, Error> {
    Ok(stop_current_task_at(chrono::Utc::now())?)
}

pub fn get_all_tasks() -> Result<HashMap<u32, Task>, Error> {
    let repo = get_repo()?;
    Ok(repo.list_tasks()?)
}

pub fn get_total_duration(
    start: chrono::DateTime<chrono::Utc>,
    end: chrono::DateTime<chrono::Utc>
) -> Result<chrono::Duration, Error> {
    let repo = get_repo()?;

    let mut total_duration = chrono::Duration::seconds(0);

    for task in repo.list_tasks()?.values() {
        total_duration = total_duration.add(task.duration_between(start, end));
    }

    Ok(total_duration)
}

pub fn format_duration(duration: chrono::Duration) -> String {
    let mut formatted_duration = String::new();

    if duration.num_hours() >= 24 {
        formatted_duration.push_str(format!("{}d ", duration.num_days()).as_str());
    }

    if duration.num_minutes() >= 60 {
        formatted_duration.push_str(format!("{}h ", duration.num_hours()).as_str());
    }

    if duration.num_seconds() >= 60 {
        formatted_duration.push_str(format!("{}m ", duration.num_minutes() % 60).as_str());
    }

    if duration.num_seconds() < 60 {
        formatted_duration.push_str(format!("{}s", duration.num_seconds() % 60).as_str());
    }

    formatted_duration
}