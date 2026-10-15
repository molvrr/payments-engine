#![allow(unused)]
use payments_engine::{engine::Engine, operation::Operation};

// TODO: Don't box every single error.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(csv_file) = std::env::args().nth(1) {
        let file = std::fs::File::open(csv_file)?;
        let mut reader = csv::Reader::from_reader(file);
        let mut engine = Engine::new();

        for result in reader.deserialize::<Operation>() {
            if let Ok(transaction) = result {
                engine.process(transaction);
            } else {
                // TODO: Log error
            }
        }
    } else {
        eprintln!("Usage should be ./binary <transactions_file.csv>");
    }

    Ok(())
}
