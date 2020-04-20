use cargo_apk::{ApkBuilder, Error};
use cargo_subcommand::Subcommand;
use exitfailure::ExitDisplay;
use std::process::Command;

fn main() -> Result<(), ExitDisplay<Error>> {
    env_logger::init();

    let cmd = Subcommand::new("apk", |_, _| Ok(false)).map_err(|e| Error::Subcommand(e))?;
    let builder = ApkBuilder::from_subcommand(&cmd)?;

    match cmd.cmd() {
        "build" => {
            for artifact in cmd.artifacts() {
                builder.build(artifact)?;
            }
        }
        "run" => {
            if cmd.artifacts().len() == 1 {
                builder.run(&cmd.artifacts()[0])?;
            } else {
                return Err(Error::invalid_args().into());
            }
        }
        "--" => {
            builder.default()?;
        }
        "gdb" => {
            if cmd.artifacts().len() == 1 {
                builder.gdb(&cmd.artifacts()[0])?;
            } else {
                return Err(Error::invalid_args().into());
            }
        }
        "help" => {
            if let Some(arg) = cmd.args().get(0) {
                match &**arg {
                    "build" | "run" | "test" | "doc" => run_cargo(&cmd)?,
                    "gdb" => print_gdb_help(),
                    _ => print_help(),
                }
            } else {
                print_help();
            }
        }
        _ => print_help(),
    }

    Ok(())
}

fn run_cargo(cmd: &Subcommand) -> Result<(), Error> {
    Command::new("cargo")
        .arg(cmd.cmd())
        .args(cmd.args())
        .status()?;
    Ok(())
}

fn print_help() {
    println!(
        r#"cargo-apk
Helps cargo build apk's for android

USAGE:
    cargo apk [SUBCOMMAND]

SUBCOMMAND:
    build   Compiles the current package
    run     Run a binary or example of the local package
    gdb     Start a gdb session attached to an adb device with symbols loaded
"#
    );
}

fn print_gdb_help() {
    println!(
        r#"cargo-apk gdb
Start a gdb session attached to an adb device with symbols loaded

USAGE:
    cargo apk gdb
"#
    );
}
