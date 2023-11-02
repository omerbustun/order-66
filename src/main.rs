use clap::Parser;
use tokio::fs as async_fs;
use anyhow::Result;
use tracing::{info, error, Level};
use tracing_subscriber;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Serialize, Deserialize)]
enum TaskStatus {
    Pending,
    Completed,
    Expired,
    Cancelled,
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
    let (mut pending_count, mut completed_count, mut expired_count, mut cancelled_count) = (0, 0, 0, 0);
    for task in &tasks {
        match task.status {
            TaskStatus::Pending => pending_count += 1,
            TaskStatus::Completed => completed_count += 1,
            TaskStatus::Expired => expired_count += 1,
            TaskStatus::Cancelled => cancelled_count += 1,
        }
    }

    info!("Task Summary:");
    info!("Pending:   {}", pending_count);
    info!("Completed: {}", completed_count);
    info!("Expired:   {}", expired_count);
    info!("Cancelled: {}", cancelled_count);

    for task in tasks.iter_mut() {
        match task.status {
            TaskStatus::Pending => {
                let duration_until_delete = task.delete_at.signed_duration_since(Utc::now());
                info!("Duration until delete: {:?}", duration_until_delete);
                if duration_until_delete > chrono::Duration::zero() {
                    let file_path = task.file_path.clone();
                    info!("Scheduling deletion for: {}", &file_path);
                    tokio::spawn(async move {
                        info!("Waiting to delete file: {}", &file_path);
                        tokio::time::sleep(duration_until_delete.to_std().unwrap()).await;
                        info!("Attempting to delete file: {}", &file_path);
                        match async_fs::remove_file(&file_path).await {
                            Ok(_) => {
                                info!("'{}' has been deleted.", &file_path);
                            },
                            Err(e) => {
                                error!("Failed to delete file '{}': {}", &file_path, e);
                            },
                        }
                    });
                } else {
                    task.status = TaskStatus::Expired;
                }
            }
            _ => {}
        }
    }
    

    save_tasks(&tasks)?;

    if let (Some(file_path), Some(time_in_minutes)) = (opts.file_path, opts.time_in_minutes) {
        let delete_at = Utc::now() + chrono::Duration::seconds(time_in_minutes as i64 * 60);
        info!("Scheduled to delete '{}' in {} minutes.", file_path, time_in_minutes);
    
        let new_task = DeletionTask {
            file_path: file_path.clone(), 
            delete_at,
            created_at: Utc::now(),
            status: TaskStatus::Pending,
        };
        
        tasks.push(new_task);
        
        save_tasks(&tasks)?;
    }
    

    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl_c");
    Ok(())
}
