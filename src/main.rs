use std::process;

use payments_engine::{engine::Engine, operation::Operation};

fn main() {
    if let Some(csv_file) = std::env::args().nth(1) {
        let file = match std::fs::File::open(csv_file) {
            Ok(f) => f,
            Err(err) => {
                eprintln!("Could not read csv file: {err}");
                process::exit(1);
            }
        };
        let mut reader = csv::ReaderBuilder::new()
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(file);
        let mut engine = Engine::new();

        for result in reader.deserialize::<Operation>() {
            match result {
                Ok(transaction) => {
                    if let Err(err) = engine.process(transaction) {
                        eprintln!("Could not process transaction: {err:?}")
                    }
                }
                Err(err) => {
                    eprintln!("Could not parse malformed record: {err}");
                }
            }
        }

        let mut writer = csv::Writer::from_writer(std::io::stdout());
        writer
            .write_record(["client", "available", "held", "total", "locked"])
            .expect("Could not write headers");

        for (_, account) in engine.state.accounts {
            writer
                .write_record([
                    account.id.to_string(),
                    account.available.to_string(),
                    account.held.to_string(),
                    account.total().to_string(),
                    account.locked.to_string(),
                ])
                .expect("Could not write headers");
        }

        writer.flush().expect("Could not flush");
    } else {
        eprintln!("Usage should be ./binary <transactions_file.csv>");
    }
}
