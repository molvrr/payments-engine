use std::io::{BufWriter, Write};
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
    // Buffered so that many rejected rows don't cost one syscall each.
    let mut errors = BufWriter::new(std::io::stderr().lock());

    for result in csv_io::read_operations(csv_file)? {
        // Failing to report a rejected row must not stop processing, so
        // stderr write errors are deliberately ignored.
        let _ = match result {
            Ok(operation) => match engine.process(operation) {
                Ok(()) => Ok(()),
                Err(err) => writeln!(errors, "Could not process transaction: {err}"),
            },
            Err(err) => writeln!(errors, "Could not parse malformed record: {err}"),
        };
    }

    let _ = errors.flush();
    csv_io::write_accounts(std::io::stdout().lock(), engine.state.accounts.values())
}
