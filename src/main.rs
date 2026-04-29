use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use csv::{Reader, ReaderBuilder};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "file-check", version = "1.0")]
struct Cli {
    #[arg(short, long, required = true)]
    file: PathBuf,

    #[arg(short, long, required = true)]
    rules: PathBuf,

    #[arg(long, default_value = "check")]
    mode: Mode,

    #[arg(long, default_value = "false", action = clap::ArgAction::SetTrue)]
    verbose: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Mode {
    Check,
    Fix,
}

fn main() -> Result<()> {
    let args: Cli = Cli::parse();
    println!("Mode: {:?}", args.mode);
    println!("Verbose: {:?}", args.verbose);

    if args.file.exists() && args.rules.exists() {
        if args.verbose {
            println!("File path: {:?}", args.file);
            println!("Rules path: {:?}", args.rules);
        }
        let reader = ReaderBuilder::new();
            
        let file_data = reader.from_path(&args.file);
        // let rules_data = (&args.rules); // the rules file is a yaml
        match file_data  {
            Ok(mut rdr) => {
                println!("Successfully read file, headers {:?}", rdr.headers().unwrap());
                println!("Rows count: {:?}", rdr.records().count());
                println!("Headers count: {:?}", rdr.headers().unwrap().iter().count());

                
            }
            Err(e) => eprintln!("Error reading file: {}", e),
        }


    } else {
        println!("Not Found {:?}", if !args.file.exists() { "file" } else { "rules" });
    }
    Ok(())
}