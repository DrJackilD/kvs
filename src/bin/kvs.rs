use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, Arg, ArgMatches, SubCommand,
};
use kvs::{KvStore, KvsError, Result, Shell};
use std::process::exit;

fn main() -> Result<()> {
    let args = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("db")
                .short("d")
                .long("db")
                .help("path to database file")
                .default_value("kvs.db"),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("get key from storage")
                .arg(
                    Arg::with_name("KEY")
                        .help("search key")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("set")
                .about("set key with given value")
                .arg(
                    Arg::with_name("KEY")
                        .help("key name")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("VALUE")
                        .help("value to set")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("remove key-value pair from storage")
                .arg(
                    Arg::with_name("KEY")
                        .help("key name")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(SubCommand::with_name("shell").about("start KVS shell"))
        .get_matches();
    let db_name = if let Some(db) = args.value_of("db") {
        db
    } else {
        "kvs.db"
    };
    match args.subcommand() {
        ("set", Some(matches)) => set_cmd(db_name, matches)?,
        ("get", Some(matches)) => get_cmd(db_name, matches)?,
        ("rm", Some(matches)) => rm_cmd(db_name, matches)?,
        ("shell", Some(matches)) => shell_cmd(db_name, matches)?,
        _ => unreachable!(),
    }
    Ok(())
}

fn set_cmd(db_name: &str, args: &ArgMatches) -> Result<()> {
    let mut store = KvStore::new(db_name)?;
    let key = args.value_of("KEY").unwrap();
    let value = args.value_of("VALUE").unwrap();
    store.set(key, value)
}

fn get_cmd(db_name: &str, args: &ArgMatches) -> Result<()> {
    let mut store = KvStore::new(db_name)?;
    let key = args.value_of("KEY").unwrap();
    let entry = match store.get(key) {
        Ok(v) => v,
        Err(err) => format!("{}", err),
    };
    println!("{}", entry);
    Ok(())
}

fn rm_cmd(db_name: &str, args: &ArgMatches) -> Result<()> {
    let mut store = KvStore::new(db_name)?;
    let key = args.value_of("KEY").unwrap();
    match store.remove(key) {
        Ok(_) => Ok(()),
        Err(KvsError::KeyNotFound) => {
            eprintln!("Key not found");
            exit(1)
        }
        Err(err) => return Err(err),
    }
}

fn shell_cmd(db_name: &str, _: &ArgMatches) -> Result<()> {
    let store = KvStore::new(db_name)?;
    let mut shell = Shell::create(store);
    match shell.start() {
        Ok(_) => Ok(()),
        Err(err) => {
            println!("{}", err);
            Ok(())
        }
    }
}
