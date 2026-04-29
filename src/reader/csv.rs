use std::path::PathBuf;

use anyhow::Result;

pub fn load(path: &PathBuf) -> Result<(Vec<String>, Vec<csv::StringRecord>)> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)?;

    let headers = reader.headers()?.iter().map(|s| s.to_string()).collect();
    let records = reader.records().collect::<Result<_, csv::Error>>()?;

    Ok((headers, records))
}