use anyhow::Result;
use assert_cmd::cargo;
use atcommand::util;
use predicates::str;

#[test]
fn add_without_session_works() -> Result<()> {
    let config_dir = tempfile::tempdir().unwrap();
    let data_dir = tempfile::tempdir().unwrap();
    let work_dir = tempfile::tempdir().unwrap();
    let file_dir = work_dir.path().join("main.cpp");
    util::echo("#include <iostream>\n", &file_dir)?;

    cargo::cargo_bin_cmd!("atc")
        .env("ATCOMMAND_CONFIG_DIR", config_dir.path())
        .env("ATCOMMAND_DATA_DIR", data_dir.path())
        .arg("config")
        .arg("add-lang")
        .arg("-l")
        .arg("cpp")
        .arg("-p")
        .arg(&file_dir)
        .arg("-i")
        .arg("6017")
        .assert()
        .success();

    cargo::cargo_bin_cmd!("atc")
        .current_dir(&work_dir)
        .env("ATCOMMAND_CONFIG_DIR", config_dir.path())
        .env("ATCOMMAND_DATA_DIR", data_dir.path())
        .arg("add")
        .arg("-c")
        .arg("abc001")
        .arg("-l")
        .arg("cpp")
        .assert()
        .success()
        .stderr(str::contains("Session"));
    // TODO: addできる仕様に変更したのでディレクトリやファイルの存在を確認する
    Ok(())
}

#[test]
fn add_lang_works() -> Result<()> {
    let config_dir = tempfile::tempdir().unwrap();
    let data_dir = tempfile::tempdir().unwrap();
    let work_dir = tempfile::tempdir().unwrap();
    let file_dir = work_dir.path().join("main.cpp");
    util::echo("#include <iostream>\n", &file_dir)?;

    cargo::cargo_bin_cmd!("atc")
        .env("ATCOMMAND_CONFIG_DIR", config_dir.path())
        .env("ATCOMMAND_DATA_DIR", data_dir.path())
        .arg("config")
        .arg("add-lang")
        .arg("-l")
        .arg("cpp")
        .arg("-p")
        .arg(&file_dir)
        .arg("-i")
        .arg("6017")
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
    Ok(())
}
