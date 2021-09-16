use assert_cmd::cargo::CommandCargoExt;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn run_empty() {
    let typefile = NamedTempFile::new().unwrap();
    let mut typemake = Command::cargo_bin("typemake").expect("Could not find and compile typemake");

    typemake.arg("--typefile").arg(typefile.path());
    assert!(typemake.status().unwrap().success());
}
