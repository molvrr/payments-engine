use std::process::ExitCode;

use payments_engine::engine::Engine;
use payments_engine::io_layer::{self, IoError};

fn main() -> ExitCode {
    let Some(csv_file) = std::env::args().nth(1) else {
        eprintln!("Usage should be ./binary <transactions_file.csv>");
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

fn run(csv_file: &str) -> Result<(), IoError> {
    let mut engine = Engine::default();

    for result in io_layer::read_operations(csv_file)? {
        match result {
            Ok(operation) => {
                if let Err(err) = engine.process(operation) {
                    eprintln!("Could not process transaction: {err}");
                }
            }
            Err(err) => eprintln!("Could not parse malformed record: {err}"),
        }
    }

    io_layer::write_accounts(std::io::stdout().lock(), engine.state.accounts.values())
}
