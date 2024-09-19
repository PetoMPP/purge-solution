use anyhow::{anyhow, Error};
use clap::Parser;
use cleaner::Cleaner;
use git::GitService;
use indicatif::MultiProgress;
use std::path::PathBuf;

mod cleaner;
mod command;
mod git;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "./")]
    path: PathBuf,
    #[arg(short, long)]
    nuget: bool,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let Args { path, nuget } = Args::parse();
    if !path.exists() {
        return Err(anyhow!("{} does not exist", path.display()));
    }

    if !path.is_dir() {
        return Err(anyhow!("{} is not a directory", path.display()));
    }

    let multibar = MultiProgress::new();
    let gitbar = multibar.add(indicatif::ProgressBar::new_spinner());
    let cleanbar = multibar.add(indicatif::ProgressBar::new_spinner());

    let start = std::time::Instant::now();
    std::env::set_current_dir(&path)?;
    let mut git = GitService::new(gitbar.clone()).await;
    git.save_working_dir().await?;

    let mut cleaner = Cleaner::new(nuget, cleanbar.clone());
    cleaner.clean(start).await?;

    git.restore_working_dir().await?;

    Ok(())
}
