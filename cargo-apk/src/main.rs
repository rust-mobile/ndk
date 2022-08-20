use cargo_apk::{ApkBuilder, Error};
use cargo_subcommand::Subcommand;
use std::process::Command;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let mut args = std::env::args();
    let mut new_args: Vec<String> = vec![];
    let mut device_name = None;

    while let Some(name) = args.next() {
        if name == "--device" {
            if let Some(dev_name_arg) = args.next() {
                device_name = Some(dev_name_arg);
            } else {
                println!("Expected device name after `--device`");
                return Err(Error::invalid_args().into());
            }
        } else {
            new_args.push(name);
        }
    }

    if let Some(device_name) = &device_name {
        println!("Running on {}", device_name);
    }

    let cmd = Subcommand::new(new_args.into_iter(), "apk", |_, _| Ok(false))
        .map_err(Error::Subcommand)?;
    let builder = ApkBuilder::from_subcommand(&cmd)?;

    match cmd.cmd() {
        "check" | "c" => builder.check()?,
        "build" | "b" => {
            for artifact in cmd.artifacts() {
                builder.build(artifact)?;
            }
        }
        "run" | "r" => {
            anyhow::ensure!(cmd.artifacts().len() == 1, Error::invalid_args());

            builder.run(&cmd.artifacts()[0], device_name)?;
        }
        "--" => {
            builder.default()?;
        }
        "gdb" => {
            anyhow::ensure!(cmd.artifacts().len() == 1, Error::invalid_args());
            builder.gdb(&cmd.artifacts()[0], device_name)?;
        }
        "help" => {
            if let Some(arg) = cmd.args().get(0) {
                match &**arg {
                    "build" | "b" | "check" | "c" | "run" | "r" | "test" | "t" | "doc" => {
                        run_cargo(&cmd)?
                    }
                    "gdb" => print_gdb_help(),
                    _ => print_help(),
                }
            } else {
                print_help();
            }
        }
        "version" => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
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
    check, c    Checks that the current package builds without creating an apk
    build, b    Compiles the current package and creates an apk
    run, r      Run a binary or example of the local package
    gdb         Start a gdb session attached to an adb device with symbols loaded
    version     Print the version of cargo-apk
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
