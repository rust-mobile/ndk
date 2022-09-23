use cargo_apk::{ApkBuilder, Error};
use cargo_subcommand::Subcommand;
use clap::Parser;

#[derive(Parser)]
struct Cmd {
    #[clap(subcommand)]
    apk: ApkCmd,
}

#[derive(clap::Subcommand)]
enum ApkCmd {
    /// Helps cargo build apks for Android
    Apk {
        #[clap(subcommand)]
        cmd: ApkSubCmd,
    },
}

#[derive(Parser)]
struct Args {
    #[clap(flatten)]
    subcommand_args: cargo_subcommand::Args,
    /// Use device with the given serial (see `adb devices`)
    #[clap(short, long)]
    device: Option<String>,
}

#[derive(clap::Subcommand)]
#[clap(trailing_var_arg = true)]
enum ApkSubCmd {
    /// Analyze the current package and report errors, but don't build object files nor an apk
    #[clap(visible_alias = "c")]
    Check {
        #[clap(flatten)]
        args: Args,
    },
    /// Compile the current package and create an apk
    #[clap(visible_alias = "b")]
    Build {
        #[clap(flatten)]
        args: Args,
    },
    /// Invoke `cargo` under the detected NDK environment
    #[clap(name = "--")]
    Ndk {
        cargo_cmd: String,
        #[clap(flatten)]
        args: Args,
    },
    /// Run a binary or example apk of the local package
    #[clap(visible_alias = "r")]
    Run {
        #[clap(flatten)]
        args: Args,
        /// Do not print or follow `logcat` after running the app
        #[clap(short, long)]
        no_logcat: bool,
    },
    /// Start a gdb session attached to an adb device with symbols loaded
    Gdb {
        #[clap(flatten)]
        args: Args,
    },
    /// Print the version of cargo-apk
    Version,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let Cmd {
        apk: ApkCmd::Apk { cmd },
    } = Cmd::parse();
    match cmd {
        ApkSubCmd::Check { args } => {
            let cmd = Subcommand::new(args.subcommand_args)?;
            let builder = ApkBuilder::from_subcommand(&cmd, args.device)?;
            builder.check()?;
        }
        ApkSubCmd::Build { args } => {
            let cmd = Subcommand::new(args.subcommand_args)?;
            let builder = ApkBuilder::from_subcommand(&cmd, args.device)?;
            for artifact in cmd.artifacts() {
                builder.build(artifact)?;
            }
        }
        ApkSubCmd::Ndk { cargo_cmd, args } => {
            let cmd = Subcommand::new(args.subcommand_args)?;
            let builder = ApkBuilder::from_subcommand(&cmd, args.device)?;
            builder.default(&cargo_cmd)?;
        }
        ApkSubCmd::Run { args, no_logcat } => {
            let cmd = Subcommand::new(args.subcommand_args)?;
            let builder = ApkBuilder::from_subcommand(&cmd, args.device)?;
            anyhow::ensure!(cmd.artifacts().len() == 1, Error::invalid_args());
            builder.run(&cmd.artifacts()[0], no_logcat)?;
        }
        ApkSubCmd::Gdb { args } => {
            let cmd = Subcommand::new(args.subcommand_args)?;
            let builder = ApkBuilder::from_subcommand(&cmd, args.device)?;
            anyhow::ensure!(cmd.artifacts().len() == 1, Error::invalid_args());
            builder.gdb(&cmd.artifacts()[0])?;
        }
        ApkSubCmd::Version => {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        }
    }
    Ok(())
}
