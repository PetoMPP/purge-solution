use crate::command::Command;
use anyhow::Error;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct GitService {
    stash_name: Option<String>,
    progress: ProgressBar,
    available: bool,
}

impl Command for GitService {
    const CMD: &'static str = "git";
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
        let Some(stash_idx) = Self::exec("stash list")
            .await?
            .lines()
            .position(|l| l.contains(stash_name))
        else {
            self.progress
                .set_style(ProgressStyle::with_template("⚠️ Git: {msg}").unwrap());
            self.progress.finish_with_message("Unable to find stash!");
            return Ok(());
        };

        self.reset_working_directory().await?;

        Self::exec(format!("stash pop --index {stash_idx}").as_str()).await?;

        self.progress
            .set_style(ProgressStyle::with_template("✔️ Git: {msg}").unwrap());
        self.progress.finish_with_message("Changes restored");

        Ok(())
    }

    async fn version() -> Result<String, Error> {
        Self::exec("version").await
    }

    async fn create_stash(&mut self) -> Result<(), Error> {
        let name = rand::random::<u32>().to_string();
        Self::exec(format!("stash -u -m {name}").as_str()).await?;
        self.stash_name = Some(name);
        Ok(())
    }

    async fn reset_working_directory(&self) -> Result<(), Error> {
        if !self.available {
            return Ok(());
        }
        Self::exec("reset --hard").await?;
        Ok(())
    }

    async fn status(&self) -> Result<Vec<String>, Error> {
        Ok(Self::exec("status -s")
            .await?
            .lines()
            .map(|l| l.to_string())
            .collect())
    }
}
