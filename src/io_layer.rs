use std::fmt;
use std::fs::File;
use std::io::{self, Write};

use crate::account::Account;
use crate::operation::Operation;

#[derive(Debug)]
pub enum IoError {
    OpenInput(io::Error),
    WriteHeaders(csv::Error),
    WriteRecord(csv::Error),
    Flush(io::Error),
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenInput(err) => write!(f, "could not open input file: {err}"),
            Self::WriteHeaders(err) => write!(f, "could not write headers: {err}"),
            Self::WriteRecord(err) => write!(f, "could not write record: {err}"),
            Self::Flush(err) => write!(f, "could not flush output: {err}"),
        }
    }
}

impl std::error::Error for IoError {}

pub fn read_operations(
    path: &str,
) -> Result<impl Iterator<Item = Result<Operation, csv::Error>>, IoError> {
    let file = File::open(path).map_err(IoError::OpenInput)?;
    let reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(file);

    Ok(reader.into_deserialize())
}

pub fn write_accounts<'a, W: Write>(
    output: W,
    accounts: impl IntoIterator<Item = &'a Account>,
) -> Result<(), IoError> {
    let mut writer = csv::Writer::from_writer(output);

    writer
        .write_record(["client", "available", "held", "total", "locked"])
        .map_err(IoError::WriteHeaders)?;

    for account in accounts {
        writer
            .write_record([
                account.id.to_string(),
                account.available.to_string(),
                account.held.to_string(),
                account.total().to_string(),
                account.locked.to_string(),
            ])
            .map_err(IoError::WriteRecord)?;
    }

    writer.flush().map_err(IoError::Flush)
}
