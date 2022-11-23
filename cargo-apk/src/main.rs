use std::collections::HashMap;

use cargo_apk::{ApkBuilder, Error};
use cargo_subcommand::Subcommand;
use clap::{CommandFactory, FromArgMatches, Parser};

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

#[derive(Clone, Debug, Eq, PartialEq, Parser)]
#[group(skip)]
struct Args {
    #[clap(flatten)]
    subcommand_args: cargo_subcommand::Args,
    /// Use device with the given serial (see `adb devices`)
    #[clap(short, long)]
    device: Option<String>,
}

#[derive(clap::Subcommand)]
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
        /// `cargo` subcommand to run
        cargo_cmd: String,

        // This struct will be filled up later by arguments that are intermixed
        // with unknown args and ended up in `cargo_args` below.
        #[clap(flatten)]
        args: Args,

        /// Arguments passed to cargo. Some arguments will be used to configure
        /// the environment similar to other `cargo apk` commands
        #[clap(trailing_var_arg = true, allow_hyphen_values = true)]
        cargo_args: Vec<String>,
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

fn split_apk_and_cargo_args(mut args: Args, input: Vec<String>) -> (Args, Vec<String>) {
    // Clap doesn't support parsing unknown args properly:
    // https://github.com/clap-rs/clap/issues/1404
    // https://github.com/clap-rs/clap/issues/4498
    // Introspect the `Args` struct and extract every known arg, and whether it takes a value. Use
    // this information to separate out known args from unknown args, and re-parse all the known
    // args into an existing `args: Args` struct instance.

    let known_args_taking_value = Args::command()
        .get_arguments()
        .flat_map(|arg| {
            assert!(!arg.is_positional());
            arg.get_short_and_visible_aliases()
                .iter()
                .flat_map(|shorts| shorts.iter().map(|short| format!("-{}", short)))
                .chain(
                    arg.get_long_and_visible_aliases()
                        .iter()
                        .flat_map(|longs| longs.iter().map(|short| format!("--{}", short))),
                )
                .map(|arg_str| (arg_str, arg.get_action().takes_values()))
                // Collect to prevent lifetime issues on temporaries created above
                .collect::<Vec<_>>()
        })
        .collect::<HashMap<_, _>>();

    #[derive(Debug, Default)]
    struct SplitArgs {
        apk_args: Vec<String>,
        cargo_args: Vec<String>,
        next_takes_value: bool,
    }

    let split_args = input
        .into_iter()
        .fold(SplitArgs::default(), |mut split_args, elem| {
            let known_arg = known_args_taking_value.get(&elem);
            if known_arg.is_some() || split_args.next_takes_value {
                // Recognized arg or value for previously recognized arg
                split_args.apk_args.push(elem)
            } else {
                split_args.cargo_args.push(elem)
            }

            split_args.next_takes_value = known_arg.copied().unwrap_or(false);
            split_args
        });

    let m = Args::command()
        .no_binary_name(true)
        .get_matches_from(&split_args.apk_args);
    args.update_from_arg_matches(&m).unwrap();
    (args, split_args.cargo_args)
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
        ApkSubCmd::Ndk {
            cargo_cmd,
            args,
            cargo_args,
        } => {
            let (args, cargo_args) = split_apk_and_cargo_args(args, cargo_args);

            let cmd = Subcommand::new(args.subcommand_args)?;
            let builder = ApkBuilder::from_subcommand(&cmd, args.device)?;
            builder.default(&cargo_cmd, &cargo_args)?;
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

#[test]
fn test_split_apk_and_cargo_args() {
    // Set up a default because cargo-subcommand doesn't derive/implement Default
    let args_default = Args::parse_from(std::iter::empty::<&str>());

    assert_eq!(
        split_apk_and_cargo_args(args_default.clone(), vec!["--quiet".to_string()]),
        (
            Args {
                subcommand_args: cargo_subcommand::Args {
                    quiet: true,
                    ..args_default.subcommand_args.clone()
                },
                ..args_default.clone()
            },
            vec![]
        )
    );

    assert_eq!(
        split_apk_and_cargo_args(
            args_default.clone(),
            vec!["unrecognized".to_string(), "--quiet".to_string()]
        ),
        (
            Args {
                subcommand_args: cargo_subcommand::Args {
                    quiet: true,
                    ..args_default.subcommand_args.clone()
                },
                ..args_default.clone()
            },
            vec!["unrecognized".to_string()]
        )
    );

    assert_eq!(
        split_apk_and_cargo_args(
            args_default.clone(),
            vec!["--unrecognized".to_string(), "--quiet".to_string()]
        ),
        (
            Args {
                subcommand_args: cargo_subcommand::Args {
                    quiet: true,
                    ..args_default.subcommand_args.clone()
                },
                ..args_default.clone()
            },
            vec!["--unrecognized".to_string()]
        )
    );

    assert_eq!(
        split_apk_and_cargo_args(
            args_default.clone(),
            vec!["-p".to_string(), "foo".to_string()]
        ),
        (
            Args {
                subcommand_args: cargo_subcommand::Args {
                    package: vec!["foo".to_string()],
                    ..args_default.subcommand_args.clone()
                },
                ..args_default.clone()
            },
            vec![]
        )
    );

    assert_eq!(
        split_apk_and_cargo_args(
            args_default.clone(),
            vec![
                "-p".to_string(),
                "foo".to_string(),
                "--unrecognized".to_string(),
                "--quiet".to_string()
            ]
        ),
        (
            Args {
                subcommand_args: cargo_subcommand::Args {
                    quiet: true,
                    package: vec!["foo".to_string()],
                    ..args_default.subcommand_args.clone()
                },
                ..args_default.clone()
            },
            vec!["--unrecognized".to_string()]
        )
    );

    assert_eq!(
        split_apk_and_cargo_args(
            args_default.clone(),
            vec![
                "--no-deps".to_string(),
                "-p".to_string(),
                "foo".to_string(),
                "--unrecognized".to_string(),
                "--quiet".to_string()
            ]
        ),
        (
            Args {
                subcommand_args: cargo_subcommand::Args {
                    quiet: true,
                    package: vec!["foo".to_string()],
                    ..args_default.subcommand_args.clone()
                },
                ..args_default.clone()
            },
            vec!["--no-deps".to_string(), "--unrecognized".to_string()]
        )
    );

    assert_eq!(
        split_apk_and_cargo_args(
            args_default.clone(),
            vec![
                "--no-deps".to_string(),
                "--device".to_string(),
                "adb:test".to_string(),
                "--unrecognized".to_string(),
                "--quiet".to_string()
            ]
        ),
        (
            Args {
                subcommand_args: cargo_subcommand::Args {
                    quiet: true,
                    ..args_default.subcommand_args
                },
                device: Some("adb:test".to_string()),
            },
            vec!["--no-deps".to_string(), "--unrecognized".to_string()]
        )
    );
}
