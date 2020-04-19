#[macro_use]
extern crate clap;
use clap::{App, AppSettings};
use kvs::client::create_client;
use kvs::command::Command;
use kvs::error::{Error, ErrorKind, Result};
use kvs::protocol::Value;
use std::env;

fn main() -> Result<()> {
    let yaml = load_yaml!("client-cli.yml");

    let matches = App::from_yaml(yaml)
        .author(crate_authors!())
        .version(crate_version!())
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    let mut addr: Option<&str> = None;
    let mut command: Option<Command> = None;
    if let Some(matches) = matches.subcommand_matches("get") {
        addr = matches.value_of("addr");
        let key = matches.value_of("KEY").unwrap();
        command = Some(Command::Get(key.to_string()));
    }

    if let Some(matches) = matches.subcommand_matches("set") {
        addr = matches.value_of("addr");
        let key = matches.value_of("KEY").unwrap();
        let value = matches.value_of("VALUE").unwrap();
        command = Some(Command::Set(key.to_string(), value.to_string()));
    }

    if let Some(matches) = matches.subcommand_matches("rm") {
        addr = matches.value_of("addr");
        let key = matches.value_of("KEY").unwrap();
        command = Some(Command::Rm(key.to_string()));
    }

    if let Some(cmd) = command {
        if let Some(address) = addr {
            let mut client = match create_client(address) {
                Ok(cli) => cli,
                Err(err) => {
                    eprintln!("Connection failed. {}", err);
                    return Ok(());
                }
            };
            match client.send_cmd(cmd) {
                Ok(value) => match value {
                    Value::None => {
                        if let Some(_matches) = matches.subcommand_matches("get") {
                            println!("Key not found")
                        }
                    }
                    Value::Integer(i) => println!("{}", i),
                    Value::String(s) => println!("{}", s),
                    Value::Error(_) => return Err(Error::from(ErrorKind::KeyNotFound)),
                    _ => return Err(Error::from(ErrorKind::UnknownError)),
                },
                Err(err) => return Err(err),
            }
        }
    }
    Ok(())
}
