use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(name)
}

#[test]
fn 正常な設定ファイルとスキーマで成功する() {
    let conf_path = fixture_path("conf/sample3.conf");
    let schema_path = fixture_path("schma/sample.schema");

    let mut cmd = Command::cargo_bin("check_conf").unwrap();
    cmd.arg(conf_path).arg(schema_path).assert().success();
}

#[test]
fn エラー無視行の型不一致はバリデーション対象外になる() {
    let conf_path = fixture_path("conf/ignore_failure_invalid_retry.conf");
    let schema_path = fixture_path("schma/sample.schema");

    let mut cmd = Command::cargo_bin("check_conf").unwrap();
    cmd.arg(conf_path).arg(schema_path).assert().success();
}

#[test]
fn スキーマがconfの全パスを網羅していなくても成功する() {
    let conf_path = fixture_path("conf/sample3.conf");
    let schema_path = fixture_path("schma/partial.schema");

    let mut cmd = Command::cargo_bin("check_conf").unwrap();
    cmd.arg(conf_path).arg(schema_path).assert().success();
}

#[test]
fn 設定ファイルのパスが存在しない場合は失敗する() {
    let schema_path = fixture_path("schma/sample.schema");

    let mut cmd = Command::cargo_bin("check_conf").unwrap();
    cmd.arg("tests/data/conf/does_not_exist.conf")
        .arg(schema_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: File not found"));
}

#[test]
fn スキーマファイルのパスが存在しない場合は失敗する() {
    let conf_path = fixture_path("conf/sample3.conf");

    let mut cmd = Command::cargo_bin("check_conf").unwrap();
    cmd.arg(conf_path)
        .arg("tests/data/schma/does_not_exist.schema")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: File not found"));
}

#[test]
fn 設定値の型が不正な場合はバリデーションに失敗する() {
    let conf_path = fixture_path("conf/invalid_retry.conf");
    let schema_path = fixture_path("schma/sample.schema");

    let mut cmd = Command::cargo_bin("check_conf").unwrap();
    cmd.arg(conf_path)
        .arg(schema_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Validation errors:"))
        .stderr(predicate::str::contains("retry"));
}

#[test]
fn 設定ファイルのパスがファイルでない場合は失敗する() {
    let schema_path = fixture_path("schma/sample.schema");
    let directory_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("data");

    let mut cmd = Command::cargo_bin("check_conf").unwrap();
    cmd.arg(directory_path)
        .arg(schema_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: Path is not a file"));
}

#[test]
fn スキーマファイルのパスがファイルでない場合は失敗する() {
    let conf_path = fixture_path("conf/sample3.conf");
    let directory_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("data");

    let mut cmd = Command::cargo_bin("check_conf").unwrap();
    cmd.arg(conf_path)
        .arg(directory_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: Path is not a file"));
}

