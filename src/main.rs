use clap::Parser;
use std::fs;
use std::thread;
use std::time::Duration;

#[derive(Parser)]
struct Opts {
    /// Path to the file to be deleted
    #[clap(short, long)]
    file_path: String,
    
    /// Time in minutes after which the file will be deleted
    #[clap(short, long)]
    time_in_minutes: u64,
}

fn main() {
    // Parse the command line arguments using clap
    let opts: Opts = Opts::parse();

    // Extract file path and time in minutes
    let file_path = &opts.file_path;
    let time_in_minutes = opts.time_in_minutes;

    // Calculate the duration in milliseconds
    let duration = Duration::from_secs(time_in_minutes * 60);

    println!("Scheduled to delete '{}' in {} minutes.", file_path, time_in_minutes);

    // Sleep for the given duration
    thread::sleep(duration);

    // Attempt to delete the file
    match fs::remove_file(file_path) {
        Ok(_) => println!("'{}' has been deleted.", file_path),
        Err(e) => println!("Error deleting '{}': {}", file_path, e),
    }
}
