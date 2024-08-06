use anyhow::{anyhow, Error};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "./")]
    path: PathBuf,
    #[arg(short, long)]
    nuget: bool,
}

fn main() -> Result<(), Error> {
    let Args { path, nuget } = Args::parse();
    if !path.exists() {
        return Err(anyhow!("{} does not exist", path.display()));
    }

    if !path.is_dir() {
        return Err(anyhow!("{} is not a directory", path.display()));
    }

    Ok(())
}
