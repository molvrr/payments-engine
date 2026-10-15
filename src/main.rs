use std::process::ExitCode;

use payments_engine::Engine;
use payments_engine::csv_io::{self, CsvIoError};

fn main() -> ExitCode {
    let Some(csv_file) = std::env::args().nth(1) else {
        eprintln!("Usage should be ./payments-engine <transactions_file.csv>");
        return ExitCode::FAILURE;
    };

    match run(&csv_file) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn run(csv_file: &str) -> Result<(), CsvIoError> {
    let mut engine = Engine::default();

    for result in csv_io::read_operations(csv_file)? {
        match result {
            Ok(operation) => {
                if let Err(err) = engine.process(operation) {
                    eprintln!("Could not process transaction: {err}");
                }
            }
            Err(err) => eprintln!("Could not parse malformed record: {err}"),
        }
    }

    csv_io::write_accounts(std::io::stdout().lock(), engine.state.accounts.values())
}
