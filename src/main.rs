use crate::{package::Package, symlink::SymlinkStatus};
use clap::{Parser, Subcommand};
use std::{env, fs, path::PathBuf};

mod package;
mod symlink;

#[derive(Debug, Clone, Parser)]
struct Cli {
    #[arg(
        long_help = "Optional, the `HOME_SYMLINK_DIR` environment variable can be used to specify this argument instead."
    )]
    dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    #[command(alias = "l")]
    Link {
        #[arg(short, long)]
        force: bool,
    },
    #[command(alias = "u")]
    Unlink {
        #[arg(short, long)]
        force: bool,
    },
    #[command(alias = "s")]
    Status,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let dir = args
        .dir
        .unwrap_or(expand_home(&env::var("HOME_SYMLINK_DIR")?));

    let package_dirs = fs::read_dir(dir)?.filter_map(Result::ok);
    let mut packages = Vec::new();

    for dir in package_dirs {
        packages.push(Package::new(&dir));
    }

    match args.command {
        Command::Link { force } => {
            for pkg in &mut packages {
                for symlink in &mut pkg.symlinks {
                    if symlink.status != SymlinkStatus::Linked {
                        symlink.link(force);
                    }
                }
            }

            for pkg in &packages {
                println!("{pkg}");
            }
        }
        Command::Unlink { force } => {
            for pkg in &mut packages {
                for symlink in &mut pkg.symlinks {
                    if symlink.status != SymlinkStatus::Unlinked {
                        symlink.unlink(force);
                    }
                }
            }

            for pkg in &packages {
                println!("{pkg}");
            }
        }
        Command::Status => {
            for pkg in &packages {
                println!("{pkg}");
            }
        }
    }

    Ok(())
}

fn expand_home(path: &str) -> PathBuf {
    if path.starts_with("~/")
        && let Ok(expanded_path) = env::var("HOME").map(PathBuf::from).map(|mut home| {
            home.push(&path[2..]);
            home
        })
    {
        expanded_path
    } else {
        PathBuf::from(path)
    }
}
