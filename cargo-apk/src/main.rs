use cargo_apk::{ApkBuilder, Error};
use cargo_subcommand::{Args, Subcommand};
use clap::Parser;

#[derive(Parser)]
struct Cmd {
    #[clap(subcommand)]
    apk: ApkCmd,
}

#[derive(clap::Subcommand)]
enum ApkCmd {
    /// Helps cargo build apk's for android
    Apk {
        #[clap(subcommand)]
        cmd: ApkSubCmd,
    },
}

#[derive(Parser)]
struct SelectArgs {
    /// Use device with the given serial. See `adb devices` for a list of connected Android devices.
    device: String,
}

#[derive(clap::Subcommand)]
enum ApkSubCmd {
    /// Checks that the current package builds without creating an apk
    Check {
        #[clap(flatten)]
        args: Args,
    },
    /// Compiles the current package and creates an apk
    Build {
        #[clap(flatten)]
        args: Args,
    },
    /// Run a binary or example of the local package
    Run {
        #[clap(flatten)]
        args: Args,

        // /// "Don't print and follow `logcat` after running the application.
        // no_logcat: bool,
    },
    /// Start a gdb session attached to an adb device with symbols loaded
    Gdb {
        #[clap(flatten)]
        args: Args,
    },
    Version {},
    // TODO:
    // Test {}
    // Doc {}
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let Cmd {
        apk: ApkCmd::Apk { cmd },
    } = Cmd::parse();
    match cmd {
        ApkSubCmd::Check { args } => {
            let cmd = Subcommand::new(args)?;
            let builder = ApkBuilder::from_subcommand(&cmd)?;
            builder.check()?;
        }
        ApkSubCmd::Build { args } => {
            let cmd = Subcommand::new(args)?;
            let builder = ApkBuilder::from_subcommand(&cmd)?;
            for artifact in cmd.artifacts() {
                builder.build(artifact)?;
            }
        }
        ApkSubCmd::Run { args } => {
            let cmd = Subcommand::new(args)?;
            let builder = ApkBuilder::from_subcommand(&cmd)?;
            anyhow::ensure!(cmd.artifacts().len() == 1, Error::invalid_args());
            builder.run(&cmd.artifacts()[0])?;
        }
        ApkSubCmd::Gdb { args } => {
            let cmd = Subcommand::new(args)?;
            let builder = ApkBuilder::from_subcommand(&cmd)?;
            anyhow::ensure!(cmd.artifacts().len() == 1, Error::invalid_args());
            builder.gdb(&cmd.artifacts()[0])?;
        }
        ApkSubCmd::Version {} => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        }
    }
    Ok(())
}
