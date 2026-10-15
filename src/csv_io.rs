use std::fmt;
use std::fs::File;
use std::io::{self, Read, Write};

use crate::account::Account;
use crate::operation::Operation;

#[derive(Debug)]
pub enum CsvIoError {
    OpenInput(io::Error),
    WriteHeaders(csv::Error),
    WriteRecord(csv::Error),
    Flush(io::Error),
}

impl fmt::Display for CsvIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenInput(err) => write!(f, "could not open input file: {err}"),
            Self::WriteHeaders(err) => write!(f, "could not write headers: {err}"),
            Self::WriteRecord(err) => write!(f, "could not write record: {err}"),
            Self::Flush(err) => write!(f, "could not flush output: {err}"),
        }
    }
}

impl std::error::Error for CsvIoError {}

pub fn read_operations(
    path: &str,
) -> Result<impl Iterator<Item = Result<Operation, csv::Error>>, CsvIoError> {
    let file = File::open(path).map_err(CsvIoError::OpenInput)?;
    Ok(operations_from_reader(file))
}

/// Streams operations from any CSV source (file, socket, in-memory buffer).
pub fn operations_from_reader<R: Read>(
    input: R,
) -> impl Iterator<Item = Result<Operation, csv::Error>> {
    csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(input)
        .into_deserialize()
}

pub fn write_accounts<'a, W: Write>(
    output: W,
    accounts: impl IntoIterator<Item = &'a Account>,
) -> Result<(), CsvIoError> {
    let mut writer = csv::Writer::from_writer(output);

    writer
        .write_record(["client", "available", "held", "total", "locked"])
        .map_err(CsvIoError::WriteHeaders)?;

    for account in accounts {
        writer
            .write_record([
                account.id.to_string(),
                account.available.to_string(),
                account.held.to_string(),
                account.total().to_string(),
                account.locked.to_string(),
            ])
            .map_err(CsvIoError::WriteRecord)?;
    }

    writer.flush().map_err(CsvIoError::Flush)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Amount, ClientId};

    fn render(accounts: &[Account]) -> String {
        let mut output = Vec::new();
        write_accounts(&mut output, accounts).expect("writing to a Vec cannot fail");
        String::from_utf8(output).unwrap()
    }

    #[test]
    fn writes_only_the_header_when_there_are_no_accounts() {
        assert_eq!(render(&[]), "client,available,held,total,locked\n");
    }

    #[test]
    fn writes_balances_total_and_locked_flag() {
        let accounts = [
            Account {
                id: ClientId(1),
                available: Amount::new(15, 1),
                held: Amount::new(2500, 4),
                locked: false,
            },
            Account {
                id: ClientId(2),
                available: Amount::new(-7, 0),
                held: Amount::ZERO,
                locked: true,
            },
        ];

        assert_eq!(
            render(&accounts),
            "client,available,held,total,locked\n\
             1,1.5,0.2500,1.7500,false\n\
             2,-7,0,-7,true\n"
        );
    }

    #[test]
    fn missing_input_file_is_an_open_error() {
        let result = read_operations("this/file/does/not/exist.csv");
        assert!(matches!(result, Err(CsvIoError::OpenInput(_))));
    }
}
