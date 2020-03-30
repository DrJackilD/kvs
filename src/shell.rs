/// This module contains Shell for KVS
/// For this moment commands and interface is the same, as in CLI verison
/// More features will be added later
use crate::{Cache, Entry, KvStore, Result, Storage};
use clap::{crate_authors, crate_version, App, AppSettings, Arg, ArgMatches, SubCommand};
use std::io::{stdin, stdout, Write};

const SHELL_NEW_LINE: &str = ">>> ";

/// This is main shell instance, which constantly read user's input until get Ctrl + C or quit command
pub struct Shell<S, C> {
    db: KvStore<S, C>,
}

impl<S: Storage, C: Cache> Shell<S, C> {
    pub fn create(db: KvStore<S, C>) -> Self {
        Shell { db }
    }

    /// Start the shell
    pub fn start(&mut self) -> Result<()> {
        let mut app = create_app();
        loop {
            let mut input = String::new();
            print!("{}", SHELL_NEW_LINE);
            stdout().flush()?;
            stdin().read_line(&mut input)?;
            let args: Vec<&str> = input.trim().split_whitespace().collect();
            let res_args = app.get_matches_from_safe_borrow(args);
            match res_args {
                Ok(args) => {
                    if let Some(set_cmd_args) = args.subcommand_matches("set") {
                        self.set_cmd(set_cmd_args)?;
                    } else if let Some(get_cmd_args) = args.subcommand_matches("get") {
                        self.get_cmd(get_cmd_args)?;
                    } else if let Some(_rm_cmd_args) = args.subcommand_matches("rm") {
                        self.rm_cmd(_rm_cmd_args)?;
                    } else if args.subcommand_matches("help").is_some() {
                        app.print_long_help()?;
                        println!();
                    } else if args.subcommand_matches("exit").is_some() {
                        println!("Bye!");
                        break;
                    } else {
                        println!("Error: invalid argument");
                    }
                }
                Err(err) => println!("{}", err),
            }
        }
        Ok(())
    }

    fn set_cmd(&mut self, args: &ArgMatches) -> Result<()> {
        let key = args.value_of("KEY").unwrap();
        let value = args.value_of("VALUE").unwrap();
        self.db.set(key, value)
    }

    fn get_cmd(&mut self, args: &ArgMatches) -> Result<()> {
        let key = args.value_of("KEY").unwrap();
        let entry = self.db.get(key);
        let result = match entry {
            Ok(Entry { value: Some(v), .. }) => v,
            _ => "Key not found".to_owned(),
        };
        println!("{}", result);
        Ok(())
    }

    fn rm_cmd(&mut self, args: &ArgMatches) -> Result<()> {
        let key = args.value_of("KEY").unwrap();
        self.db.remove(key)
    }
}

fn create_app<'a>() -> App<'a, 'a> {
    App::new("KVS Shell")
        .version(crate_version!())
        .author(crate_authors!())
        .setting(AppSettings::DisableVersion)
        .setting(AppSettings::DisableHelpFlags)
        .setting(AppSettings::NoBinaryName)
        .setting(AppSettings::AllowExternalSubcommands)
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
        .subcommand(SubCommand::with_name("help").about("print help"))
        .subcommand(SubCommand::with_name("exit").about("quit shell"))
}
