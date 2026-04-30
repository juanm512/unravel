use clap::{Parser, ValueEnum};
use std::path::PathBuf;
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

    if args.file.exists() && args.rules.exists() {
        println!("File path: {:?}", args.file);
        println!("Rules path: {:?}", args.rules);
        
        let (headers, records) = reader::csv::load(&args.file)?;
        // println!("Headers: {:?}", headers);
        // println!("Records: {:?}", records);

        let rules = yaml::load(&args.rules)?;
        // println!("{:#?}", rules);

        let report = validator::validate(&headers, &records, &rules);
        println!("Validation Report:");
        println!("Total rows: {}", report.total_rows);
        println!("Total errors: {}", report.total_errors);
        println!("Rows with errors: {}", report.rows_with_errors.len());
        
        let mut row_line = 0;
        for error in report.errors.iter() {
            if !error.row.eq(&row_line) { row_line = error.row.clone(); print!("\nRow {:?}: \n", &row_line); };
            println!("\tColumn '{}': {} (Value: {:?})", error.column, error.message, error.value.clone().unwrap_or_else(|| "None".to_string()));
        }
        

        if args.mode == Mode::Fix {
            let cleaned_path = cleaner::clean(&args.file, &headers, &records, &report, args.threshold);
            if cleaned_path.is_ok() {
                println!("Cleaned file generated at: {:?}", cleaned_path);
            } else {
                println!("Cleaning aborted due to high error percentage.");
            }
        }
        
    } else {
        println!("Not Found {:?}", if !args.file.exists() { "file" } else { "rules" });
    }
    Ok(())
}