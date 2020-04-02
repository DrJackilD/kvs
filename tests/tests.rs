use assert_cmd::prelude::*;
use kvs::KvStore;
use predicates::str::contains;
use std::fs::remove_file;
use std::process::Command;

const TEST_DB_NAME: &str = "test_kvs.db";

fn clean_db() {
    if let Err(_) = remove_file(TEST_DB_NAME) {};
}

// `kvs` with no args should exit with a non-zero code.
#[test]
fn cli_no_args() {
    Command::cargo_bin("kvs").unwrap().assert().failure();
}

// `kvs -V` should print the version
#[test]
fn cli_version() {
    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["-V"])
        .assert()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}

// `kvs get <KEY>` should print "unimplemented" to stderr and exit with non-zero code
#[test]
fn cli_get() {
    clean_db();
    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["--db", TEST_DB_NAME, "get", "key1"])
        .assert()
        .success()
        .stdout(contains("Key not found"));
    clean_db();
}

// `kvs set <KEY> <VALUE>` should print "unimplemented" to stderr and exit with non-zero code
#[test]
fn cli_set() {
    clean_db();
    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["--db", TEST_DB_NAME, "set", "key1", "value1"])
        .assert()
        .success();
    clean_db();
}

// `kvs rm <KEY>` should print "unimplemented" to stderr and exit with non-zero code
#[test]
fn cli_rm() {
    clean_db();
    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["--db", TEST_DB_NAME, "rm", "key1"])
        .assert()
        .failure()
        .stderr(contains("Key not found"));
    clean_db();
}

#[test]
fn cli_invalid_get() {
    clean_db();
    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["--db", TEST_DB_NAME, "get"])
        .assert()
        .failure();

    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["--db", TEST_DB_NAME, "get", "extra", "field"])
        .assert()
        .failure();
    clean_db();
}

#[test]
fn cli_invalid_set() {
    clean_db();
    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["--db", TEST_DB_NAME, "set"])
        .assert()
        .failure();

    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["--db", TEST_DB_NAME, "set", "missing_field"])
        .assert()
        .failure();

    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["--db", TEST_DB_NAME, "set", "extra", "extra", "field"])
        .assert()
        .failure();
    clean_db();
}

#[test]
fn cli_invalid_rm() {
    clean_db();
    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["--db", TEST_DB_NAME, "rm"])
        .assert()
        .failure();

    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["--db", TEST_DB_NAME, "rm", "extra", "field"])
        .assert()
        .failure();
    clean_db();
}

#[test]
fn cli_invalid_subcommand() {
    clean_db();
    Command::cargo_bin("kvs")
        .unwrap()
        .args(&["--db", TEST_DB_NAME, "unknown", "subcommand"])
        .assert()
        .failure();
    clean_db();
}

// Should get previously stored value
#[test]
fn get_stored_value() {
    clean_db();
    let mut store = KvStore::new(TEST_DB_NAME).unwrap();

    store.set("key1", "value1").unwrap();
    store.set("key2", "value2").unwrap();

    assert_eq!(store.get("key1").unwrap(), "value1".to_owned());
    assert_eq!(store.get("key2").unwrap(), "value2".to_owned());
    clean_db();
}

// Should overwrite existent value
#[test]
fn overwrite_value() {
    clean_db();
    let mut store = KvStore::new(TEST_DB_NAME).unwrap();

    store.set("key1", "value1").unwrap();
    assert_eq!(store.get("key1").unwrap(), "value1".to_owned());

    store.set("key1", "value2").unwrap();
    assert_eq!(store.get("key1").unwrap(), "value2".to_owned());
    clean_db();
}

// Should get `None` when getting a non-existent key
#[test]
fn get_non_existent_value() {
    clean_db();
    let mut store = KvStore::new(TEST_DB_NAME).unwrap();

    store.set("key1", "value1").unwrap();
    assert!(store.get("key2").is_err());
    clean_db();
}

#[test]
fn remove_key() {
    clean_db();
    let mut store = KvStore::new(TEST_DB_NAME).unwrap();

    store.set("key1", "value1").unwrap();
    store.remove("key1").unwrap();
    assert!(store.get("key1").is_err());
    clean_db();
}
