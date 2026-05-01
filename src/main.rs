use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use std::time::Instant;
use anyhow::Result;

use crate::reader::yaml;

mod validator;
mod reader;
mod cleaner;

#[derive(Parser)]
#[command(name = "file-check", version = "1.0")]
struct Cli {
    #[arg(short, long, required = true)]
    file: PathBuf,

    #[arg(short, long, required = true)]
    rules: PathBuf,

    #[arg(long, default_value = "check")]
    mode: Mode,

    #[arg(short, long, default_value = "50.0")]
    threshold: f64,

    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq)]
enum Mode {
    Check,
    Fix,
}

fn main() -> Result<()> {
    let args: Cli = Cli::parse();
    println!("Mode: {:?}", args.mode);
    println!("Threshold: {:?}", args.threshold);
    println!("Verbose: {:?}", args.verbose);

    if args.file.exists() && args.rules.exists() {
        println!("File path: {:?}", args.file);
        println!("Rules path: {:?}", args.rules);
        
        let t = Instant::now();
        let (headers, records) = reader::csv::load(&args.file)?;
        let read_time = t.elapsed();
        println!("File loaded. Headers: {:?}, Records: {}", headers, records.len());

        let rules = yaml::load(&args.rules)?;

        let t = Instant::now();
        let report = validator::validate(&headers, &records, &rules);
        let validate_time = t.elapsed();
        println!("Validation Report:");
        println!("Total rows: {}", report.total_rows);
        println!("Total errors: {}", report.total_errors);
        println!("Rows with errors: {}", report.rows_with_errors.len());
        
        let mut row_line = 0;
        for error in report.errors.iter() {
            if !error.row.eq(&row_line) { row_line = error.row.clone(); print!("\nRow {:?}: \n", &row_line); };
            println!("\tColumn '{}': {} (Value: {:?})", error.column, error.message, error.value.clone().unwrap_or_else(|| "None".to_string()));
        }
        

        let mut clean_time: Option<std::time::Duration> = None;
        if args.mode == Mode::Fix {
            let t = Instant::now();
            let cleaned_path = cleaner::clean(&args.file, &headers, &records, &report, args.threshold);
            if cleaned_path.is_ok() {
                println!("Cleaned file generated at: {:?}", cleaned_path);
            } else {
                println!("Cleaning aborted due to high error percentage.");
            }
            clean_time = Some(t.elapsed());
        }
        
        if args.verbose {
            println!("  read:     {:?}", read_time);
            println!("  validate: {:?}", validate_time);
                if let Some(ct) = clean_time {                                                                                                                               
                    println!("  clean:    {:?}", ct);
                }
        }

    } else {
        println!("Not Found {:?}", if !args.file.exists() { "file" } else { "rules" });
    }
    Ok(())
}