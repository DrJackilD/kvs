use clap::{
    crate_authors, crate_description, crate_name, crate_version, App, Arg, ArgMatches, SubCommand,
};
use kvs::{KvStore, Result, Shell};

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
    if let Some(set_cmd_args) = args.subcommand_matches("set") {
        set_cmd(db_name, set_cmd_args)?;
    } else if let Some(get_cmd_args) = args.subcommand_matches("get") {
        get_cmd(db_name, get_cmd_args)?;
    } else if let Some(rm_cmd_args) = args.subcommand_matches("rm") {
        rm_cmd(db_name, rm_cmd_args)?;
    } else if let Some(shell_cmd_args) = args.subcommand_matches("shell") {
        shell_cmd(db_name, shell_cmd_args)?;
    } else {
        panic!("unrecognized command")
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
    store.remove(key)
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
