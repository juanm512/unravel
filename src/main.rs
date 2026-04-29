use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use anyhow::Result;

use crate::reader::yaml;

mod reader;

#[derive(Parser)]
#[command(name = "file-check", version = "1.0")]
struct Cli {
    #[arg(short, long, required = true)]
    file: PathBuf,

    #[arg(short, long, required = true)]
    rules: PathBuf,

    #[arg(long, default_value = "check")]
    mode: Mode,

    #[arg(long, default_value = "false")]
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
        let (headers, records) = reader::csv::load(&args.file)?;
        // let rules_data = (&args.rules); // the rules file is a yaml
        println!("Headers: {:?}", headers);
        println!("Records: {:?}", records);


        let rules = yaml::load(&args.rules)?;
        println!("{:#?}", rules);


    } else {
        println!("Not Found {:?}", if !args.file.exists() { "file" } else { "rules" });
    }
    Ok(())
}