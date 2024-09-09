use anyhow::Error;
use indicatif::{ProgressBar, ProgressStyle};
use std::{process::Output, time::Duration};
use tokio::process::Command;

pub struct GitService {
    stash_name: Option<String>,
    progress: ProgressBar,
    available: bool,
}

impl GitService {
    pub async fn new(progress: ProgressBar) -> Self {
        progress.set_style(ProgressStyle::with_template("{spinner} Git: {msg}").unwrap());
        progress.enable_steady_tick(Duration::from_millis(100));

        if let Ok(version) = Self::version().await {
            progress.set_message(format!("Git version: {}", version));

            return Self {
                stash_name: None,
                progress,
                available: true,
            };
        }

        progress.set_message("Git not found!");
        Self {
            stash_name: None,
            progress,
            available: false,
        }
    }

    pub async fn save_working_dir(&mut self) -> Result<(), Error> {
        if !self.available {
            return Ok(());
        }
        match self.status().await?.len() {
            0 => self.progress.set_message("No changes found"),
            c => {
                self.create_stash().await?;
                self.progress.set_message(format!("Stashed {} changes.", c));
            }
        }
        Ok(())
    }

    pub async fn restore_working_dir(&self) -> Result<(), Error> {
        if !self.available {
            return Ok(());
        }
        let Some(stash_name) = &self.stash_name else {
            self.reset_working_directory().await?;
            return Ok(());
        };
        let Some(stash_idx) =
            String::from_utf8_lossy(&GitCommand::new("stash list").exec().await?.stdout)
                .to_string()
                .lines()
                .position(|l| l.contains(stash_name))
        else {
            self.progress
                .set_style(ProgressStyle::with_template("⚠️ Git: {msg}").unwrap());
            self.progress.finish_with_message("Unable to find stash!");
            return Ok(());
        };

        self.reset_working_directory().await?;

        GitCommand::new(format!("stash pop --index {stash_idx}").as_str())
            .exec()
            .await?;

        self.progress
            .set_style(ProgressStyle::with_template("✔️ Git: {msg}").unwrap());
        self.progress.finish_with_message("Changes restored");

        Ok(())
    }

    async fn version() -> Result<String, Error> {
        Ok(String::from_utf8_lossy(&GitCommand::new("version").exec().await?.stdout).to_string())
    }

    async fn create_stash(&mut self) -> Result<(), Error> {
        let name = rand::random::<u32>().to_string();
        GitCommand::new(format!("stash -u -m {name}").as_str())
            .exec()
            .await?;
        self.stash_name = Some(name);
        Ok(())
    }

    async fn reset_working_directory(&self) -> Result<(), Error> {
        if !self.available {
            return Ok(());
        }
        GitCommand::new("reset --hard").exec().await?;
        Ok(())
    }

    async fn status(&self) -> Result<Vec<String>, Error> {
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
