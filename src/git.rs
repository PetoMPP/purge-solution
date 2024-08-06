use std::process::Output;

use anyhow::Error;
use tokio::process::Command;

pub struct GitCommand {
    command: Command,
}

impl GitCommand {
    pub fn version() -> Self {
        Self::from_cmd("version")
    }

    pub fn create_stash() -> Self {
        todo!()
    }

    pub fn pop_stash() -> Self {
        todo!()
    }

    pub fn reset_working_directory() -> Self {
        todo!()
    }

    pub async fn exec(&mut self) -> Result<Output, Error> {
        Ok(self.command.spawn()?.wait_with_output().await?)
    }

    fn from_cmd(cmd: &str) -> Self {
        let mut command = Command::new("git");
        command.args(cmd.split_whitespace());
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        Self { command }
    }
}
