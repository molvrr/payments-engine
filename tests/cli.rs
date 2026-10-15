//! End-to-end tests: run the real binary the way the spec does
//! (`payments-engine transactions.csv > accounts.csv`) against the fixtures
//! in `tests/fixtures/<case>/{input,expected}.csv`.

use std::process::{Command, Output};

use rust_decimal::Decimal;

const HEADER: [&str; 5] = ["client", "available", "held", "total", "locked"];

fn run_engine(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_payments-engine"))
        .args(args)
        .output()
        .expect("failed to run the payments-engine binary")
}

fn fixture(case: &str, file: &str) -> String {
    format!(
        "{}/tests/fixtures/{case}/{file}",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// Normalizes a decimal so "1.5000", "1.5" and " 1.5" compare equal, and
/// checks the spec's four-decimal precision.
fn normalize_decimal(value: &str) -> String {
    let decimal: Decimal = value
        .parse()
        .unwrap_or_else(|err| panic!("{value:?} is not a decimal: {err}"));
    assert!(
        decimal.scale() <= 4,
        "{value} has more than four decimal places"
    );
    decimal.normalize().to_string()
}

/// Canonical form of an accounts CSV. Per the spec, spacing, decimal
/// formatting of round values and row order do not matter.
fn canonical_accounts(csv_text: &str) -> Vec<[String; 5]> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(csv_text.as_bytes());

    let header: Vec<&str> = reader.headers().expect("missing header").iter().collect();
    assert_eq!(header, HEADER);

    let mut rows: Vec<[String; 5]> = reader
        .records()
        .map(|record| {
            let record = record.expect("malformed output row");
            let locked = &record[4];
            assert!(
                locked == "true" || locked == "false",
                "locked must be a bool, got {locked:?}"
            );
            [
                record[0].to_string(),
                normalize_decimal(&record[1]),
                normalize_decimal(&record[2]),
                normalize_decimal(&record[3]),
                locked.to_string(),
            ]
        })
        .collect();
    rows.sort();
    rows
}

#[track_caller]
fn assert_fixture(case: &str) {
    let output = run_engine(&[&fixture(case, "input.csv")]);
    assert!(
        output.status.success(),
        "engine failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("stdout must be UTF-8");
    let expected =
        std::fs::read_to_string(fixture(case, "expected.csv")).expect("missing expected.csv");

    assert_eq!(
        canonical_accounts(&actual),
        canonical_accounts(&expected),
        "\nactual output:\n{actual}"
    );
}

#[test]
fn spec_example() {
    assert_fixture("spec_example");
}

#[test]
fn dispute_and_resolve() {
    assert_fixture("dispute_resolve");
}

#[test]
fn chargeback_locks_the_account() {
    assert_fixture("chargeback");
}

#[test]
fn partner_errors_are_ignored() {
    assert_fixture("partner_errors");
}

#[test]
fn malformed_rows_are_skipped() {
    assert_fixture("malformed_rows");
}

#[test]
fn four_decimal_precision() {
    assert_fixture("precision");
}

#[test]
fn rejected_rows_are_reported_on_stderr_not_stdout() {
    let output = run_engine(&[&fixture("malformed_rows", "input.csv")]);
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(stdout.starts_with("client,available,held,total,locked\n"));
    assert!(
        stderr.contains("Could not parse malformed record"),
        "stderr: {stderr}"
    );
}

#[test]
fn missing_argument_fails_with_usage() {
    let output = run_engine(&[]);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage"));
}

#[test]
fn missing_input_file_fails() {
    let output = run_engine(&["does/not/exist.csv"]);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("could not open input file"));
}
