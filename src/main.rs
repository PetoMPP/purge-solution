use anyhow::{anyhow, Error};
use clap::Parser;
use git::{GitCommand, GitService};
use std::path::PathBuf;

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

    std::env::set_current_dir(&path)?;
    let mut git = GitService::new().await;
    if let Some(git) = &mut git {
        match git.status().await?.len() {
            0 => println!("No changes found"),
            c => {
                git.create_stash().await?;
                println!("Stashed {} changes.", c);
            }
        }
    }

    // do stuff

    if let Some(git) = &git {
        git.reset_working_directory().await?;
        git.pop_stash().await?;
        println!("Changes restored.");
    }

    Ok(())
}
