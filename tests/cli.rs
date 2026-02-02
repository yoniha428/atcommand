use assert_cmd::cargo;
use predicates::str;
use std::{fs, io::Write, path::Path};

#[test]
fn config_works() {
    let config_dir = tempfile::tempdir().unwrap();
    let data_dir = tempfile::tempdir().unwrap();
    let work_dir = tempfile::tempdir().unwrap();
    let file_dir = work_dir.path().join("main.cpp");
    echo("#include <iostream>\n", &file_dir);

    cargo::cargo_bin_cmd!("atc")
        .env("ATCOMMAND_CONFIG_DIR", config_dir.path())
        .env("ATCOMMAND_DATA_DIR", data_dir.path())
        .arg("config")
        .arg("add-lang")
        .arg("-l")
        .arg("cpp")
        .arg("-p")
        .arg(&file_dir)
        .assert()
        .success();

    cargo::cargo_bin_cmd!("atc")
        .env("ATCOMMAND_CONFIG_DIR", config_dir.path())
        .env("ATCOMMAND_DATA_DIR", data_dir.path())
        .arg("config")
        .arg("lang-list")
        .assert()
        .success()
        .stdout(str::contains("cpp"));
}

fn echo(s: &str, path: &Path) {
    let mut f = fs::File::create(path)
        .unwrap_or_else(|_| panic!("Failed to open {}", path.to_string_lossy()));

    f.write_all(s.as_bytes())
        .unwrap_or_else(|_| panic!("Failed to write to {}", path.to_string_lossy()));
}
