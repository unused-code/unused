use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn doctor_works() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("unused")?;

    cmd.arg("doctor");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Unused Doctor"))
        .stdout(predicate::str::contains(
            "Are tokens found in the application?",
        ));

    Ok(())
}

#[test]
fn harsh_triggers_failure() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("unused")?;

    cmd.arg("--harsh");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn token_search_successful() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("unused")?;

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Tokens found:"));

    Ok(())
}
