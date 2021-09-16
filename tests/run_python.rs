use assert_cmd::cargo::CommandCargoExt;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn run_python() {
    let mut typefile = NamedTempFile::new().unwrap();
    writeln!(
        typefile,
        "i = 5
i += 1
assert i == 6
print('success')"
    )
    .unwrap();

    let mut typemake = Command::cargo_bin("typemake").expect("Could not find and compile typemake");

    typemake.arg("--typefile").arg(typefile.path());
    assert!(typemake.status().unwrap().success());
}
