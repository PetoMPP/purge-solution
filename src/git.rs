use anyhow::{anyhow, Error};
use std::process::Output;
use tokio::process::Command;

#[derive(Default)]
pub struct GitService {
    stash_name: Option<String>,
}

impl GitService {
    pub async fn new() -> Option<Self> {
        match Self::version().await {
            Err(_) => {
                println!("No git found!");
                None
            }
            Ok(v) => {
                println!("Using {}", v);
                Some(Self::default())
            }
        }
    }

    pub async fn version() -> Result<String, Error> {
        Ok(String::from_utf8_lossy(&GitCommand::new("version").exec().await?.stdout).to_string())
    }

    pub async fn create_stash(&mut self) -> Result<(), Error> {
        let name = rand::random::<u32>().to_string();
        GitCommand::new(format!("stash -u -m {name}").as_str())
            .exec()
            .await?;
        self.stash_name = Some(name);
        Ok(())
    }

    pub async fn pop_stash(&self) -> Result<(), Error> {
        let Some(stash_name) = &self.stash_name else {
            return Err(anyhow!("No stash was saved!"));
        };
        let stash_idx =
            String::from_utf8_lossy(&GitCommand::new("stash list").exec().await?.stdout)
                .to_string()
                .lines()
                .position(|l| l.contains(stash_name))
                .ok_or(anyhow!("Unable to find stash!"))?;

        GitCommand::new(format!("stash pop --index {stash_idx}").as_str())
            .exec()
            .await?;

        Ok(())
    }

    pub async fn reset_working_directory(&self) -> Result<(), Error> {
        GitCommand::new("reset --hard").exec().await?;
        Ok(())
    }

    pub async fn status(&self) -> Result<Vec<String>, Error> {
        Ok(
            String::from_utf8_lossy(&GitCommand::new("status -s").exec().await?.stdout)
                .to_string()
                .lines()
                .map(|l| l.to_string())
                .collect(),
        )
    }
}

pub struct GitCommand {
    command: Command,
}

impl GitCommand {
    pub fn new(cmd: &str) -> Self {
        let mut command = Command::new("git");
        command.args(cmd.split_whitespace());
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        Self { command }
    }

    pub async fn exec(&mut self) -> Result<Output, Error> {
        Ok(self.command.spawn()?.wait_with_output().await?)
    }
}
