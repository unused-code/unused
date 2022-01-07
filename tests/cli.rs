use assert_cmd::prelude::*; // Add methods on commands
use assert_fs::prelude::*;
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
    let file = assert_fs::NamedTempFile::new("tags")?;
    file.write_str("Alias	../crates/read_ctags/src/token_kind.rs	/^    Alias,$/;\"	e	enum:TokenKind")?;
    file.write_file(std::path::Path::new("tags"))?;

    let mut cmd = Command::cargo_bin("unused")?;

    cmd.arg("--harsh");
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("Tokens found:"));

    Ok(())
}

#[test]
fn token_search_successful() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("tags")?;
    file.write_str("Alias	../crates/read_ctags/src/token_kind.rs	/^    Alias,$/;\"	e	enum:TokenKind")?;

    let mut cmd = Command::cargo_bin("unused")?;

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Tokens found:"));

    Ok(())
}
