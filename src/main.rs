use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path, sync::{Arc, Mutex}};
use tokio::fs as async_fs;
use tracing::{error, info, Level};
use tracing_subscriber;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum TaskStatus {
    Pending,
    Completed,
    Expired,
    Cancelled,
    Failed,
}

#[derive(Serialize, Deserialize)]
struct DeletionTask {
    file_path: String,
    delete_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    status: TaskStatus,
}

#[derive(Parser)]
struct Opts {
    #[clap(short, long)]
    file_path: Option<String>,

    #[clap(short, long)]
    time_in_minutes: Option<u64>,
}

fn save_tasks(tasks: &[DeletionTask]) -> io::Result<()> {
    let data = serde_json::to_string(&tasks)?;
    fs::write("tasks.json", data.as_bytes())?;
    Ok(())
}

fn load_tasks() -> io::Result<Vec<DeletionTask>> {
    let path = Path::new("tasks.json");
    let mut tasks = Vec::new();

    if path.exists() {
        let data = fs::read_to_string(path)?;
        tasks = serde_json::from_str(&data)?;
    }

    Ok(tasks)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let opts: Opts = Opts::parse();

    let mut tasks = load_tasks()?;

    // Task summary
    let (mut pending_count, mut completed_count, mut expired_count, mut cancelled_count, mut failed_count) =
        (0, 0, 0, 0, 0);
    for task in &tasks {
        match task.status {
            TaskStatus::Pending => pending_count += 1,
            TaskStatus::Completed => completed_count += 1,
            TaskStatus::Expired => expired_count += 1,
            TaskStatus::Cancelled => cancelled_count += 1,
            TaskStatus::Failed => failed_count += 1,
        }
    }

    info!("Task Summary:");
    info!("Pending:   {}", pending_count);
    info!("Completed: {}", completed_count);
    info!("Expired:   {}", expired_count);
    info!("Cancelled: {}", cancelled_count);
    info!("Cancelled: {}", failed_count);


    if let (Some(file_path), Some(time_in_minutes)) = (opts.file_path, opts.time_in_minutes) {
        let delete_at = Utc::now() + chrono::Duration::seconds(time_in_minutes as i64 * 60);
        info!(
            "Scheduled to delete '{}' in {} minutes.",
            file_path, time_in_minutes
        );

        let new_task = DeletionTask {
            file_path: file_path.clone(),
            delete_at,
            created_at: Utc::now(),
            status: TaskStatus::Pending,
        };

        tasks.push(new_task);

        save_tasks(&tasks)?;
    }

    let tasks_status = Arc::new(Mutex::new(Vec::new()));

    for task in tasks.iter() {
        match task.status {
            TaskStatus::Pending => {
                let duration_until_delete = task.delete_at.signed_duration_since(Utc::now());
                info!(
                    "Duration until delete for '{}': {:?}",
                    task.file_path, duration_until_delete
                );
                if duration_until_delete > chrono::Duration::zero() {
                    // The task is pending and not expired
                    let file_path = task.file_path.clone();
                    let tasks_status = tasks_status.clone();
                    tokio::spawn(async move {
                        info!("Waiting to delete file: {}", &file_path);
                        tokio::time::sleep(duration_until_delete.to_std().unwrap()).await;
                        info!("Attempting to delete file: {}", &file_path);
                        if async_fs::remove_file(&file_path).await.is_ok() {
                            info!("'{}' has been deleted.", &file_path);
                            let mut statuses = tasks_status.lock().unwrap();
                            statuses.push((file_path, TaskStatus::Completed));
                        } else {
                            error!("Failed to delete file '{}'", &file_path);
                            let mut statuses = tasks_status.lock().unwrap();
                            statuses.push((file_path, TaskStatus::Failed));
                        }
                    });
                } else {
                    info!(
                        "Task for '{}' has expired and will not be processed.",
                        task.file_path
                    );
                    let mut statuses = tasks_status.lock().unwrap();
                    statuses.push((task.file_path.clone(), TaskStatus::Expired));
                }
            }
            _ => {}
        }
    }

    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl_c");

    {
        let shared_statuses = tasks_status.lock().unwrap();
        for (index, task) in tasks.iter_mut().enumerate() {
            if let Some((_, status)) = shared_statuses.get(index) {
                task.status = status.clone();
            }
        }
    }

    save_tasks(&tasks)?;

    Ok(())
}
