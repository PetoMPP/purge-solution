pub trait Command {
    const CMD: &'static str;
    async fn exec(args: &str) -> Result<String, anyhow::Error> {
        let mut command = tokio::process::Command::new(Self::CMD);
        command.args(args.split_whitespace());
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        let output = command.spawn()?.wait_with_output().await?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "{} failed with: {}",
                Self::CMD,
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
