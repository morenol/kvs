#[macro_use]
extern crate clap;
use clap::{App,AppSettings};
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


    if let Some(_matches) = matches.subcommand_matches("get") {
        unimplemented!("set is unimplemented");
    }

    if let Some(_matches) = matches.subcommand_matches("set") {
        unimplemented!("set is unimplemented");
    }

    if let Some(_matches) = matches.subcommand_matches("rm") {
        unimplemented!("set is unimplemented");
    }


}

