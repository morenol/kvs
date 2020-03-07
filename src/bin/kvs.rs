#[macro_use]
extern crate clap;
use clap::{App,AppSettings};

use kvs::KvStore;
use std::env;


fn main() {
    let yaml = load_yaml!("cli.yml");
    

    let matches = App::from_yaml(yaml)
                            .author(crate_authors!())
                            .version(crate_version!())
                            .about(env!("CARGO_PKG_DESCRIPTION"))
                            .setting(AppSettings::DisableHelpSubcommand)
                            .setting(AppSettings::SubcommandRequiredElseHelp)
                            .get_matches();


    if let Some(matches) = matches.subcommand_matches("get") {
        let key = matches.value_of("KEY").unwrap();
        let kvs = KvStore::new();
        kvs.get(key.to_string());
    }

    if let Some(matches) = matches.subcommand_matches("set") {
        let key = matches.value_of("KEY").unwrap();
        let value = matches.value_of("VALUE").unwrap();
        let mut kvs = KvStore::new();
        kvs.set(key.to_string(), value.to_string());
    }

    if let Some(matches) = matches.subcommand_matches("rm") {
        let key = matches.value_of("KEY").unwrap();
        let mut kvs = KvStore::new();
        kvs.remove(key.to_string());
    }


}

