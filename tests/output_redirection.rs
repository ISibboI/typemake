use assert_cmd::cargo::CommandCargoExt;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn run_python() {
    let marker1 = "caty454i5xrxere4,lyertc";
    let marker2 = "5f6c8mtek754xekjr4qy054";
    let mut typefile = NamedTempFile::new().unwrap();
    writeln!(
        typefile,
        "
import sys
print('{}')
print('{}', file = sys.stderr)"
    , marker1, marker2)
    .unwrap();

    let mut typemake = Command::cargo_bin("typemake").expect("Could not find and compile typemake");

    typemake.arg("--typefile").arg(typefile.path());
    let output = typemake.output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Rough checking if the marker strings are appearing with a log message.
    assert!(!stdout.starts_with(marker1) && !stdout.contains(&format!("\n{}", marker1)));
    assert!(!stderr.starts_with(marker2) && !stderr.contains(&format!("\n{}", marker2)));
}
