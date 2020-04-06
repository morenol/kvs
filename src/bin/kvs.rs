#[macro_use]
extern crate clap;
use clap::{App, AppSettings};
use kvs::{KvStore, Result};
use std::env;

fn main() -> Result<()> {
    let yaml = load_yaml!("cli.yml");

    let matches = App::from_yaml(yaml)
        .author(crate_authors!())
        .version(crate_version!())
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("get") {
        let storage = KvStore::open(".")?;
        let key = matches.value_of("KEY").unwrap();
        let res = storage.get(key.to_string())?;
        match res {
            Some(value) => println!("{}", value),
            None => println!("Key not found"),
        };
    }

    if let Some(matches) = matches.subcommand_matches("set") {
        let mut storage = KvStore::open(".")?;
        let key = matches.value_of("KEY").unwrap();
        let value = matches.value_of("VALUE").unwrap();
        storage.set(key.to_string(), value.to_string())?;
    }

    if let Some(matches) = matches.subcommand_matches("rm") {
        let mut storage = KvStore::open(".")?;
        let key = matches.value_of("KEY").unwrap();
        let res = storage.remove(key.to_string());
        return match res {
            Ok(_) => Ok(()),
            Err(err) => {
                println!("{}", err.to_string());
                Err(err)
            }
        };
    }
    Ok(())
}
