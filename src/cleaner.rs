use anyhow::Error;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::{future::Future, path::PathBuf, pin::Pin, time::Duration};
use tokio::io::AsyncWriteExt;

pub struct Cleaner {
    pub nuget_patterns: Option<Vec<String>>,
    result: CleanerResult,
    progress: ProgressBar,
}

#[derive(Default, Clone, Copy)]
struct CleanerResult {
    directories: usize,
    files: usize,
}

impl std::ops::AddAssign for CleanerResult {
    fn add_assign(&mut self, rhs: Self) {
        self.directories += rhs.directories;
        self.files += rhs.files;
    }
}

impl Cleaner {
    pub fn new(nuget_patterns: Option<Vec<String>>, progress: ProgressBar) -> Self {
        progress.set_style(ProgressStyle::with_template("{spinner} Cleaning: {msg}").unwrap());
        progress.enable_steady_tick(Duration::from_millis(100));

        Self {
            nuget_patterns,
            result: CleanerResult::default(),
            progress,
        }
    }

    pub async fn clean(&mut self, start: std::time::Instant) -> Result<(), Error> {
        self.result = Self::clean_bin_obj(
            PathBuf::from("."),
            CleanerResult::default(),
            self.progress.clone(),
        )
        .await;
        if let Some(patterns) = self.nuget_patterns.clone() {
            self.result += Self::clean_nuget(patterns, self.progress.clone()).await;
        }

        self.progress
            .set_style(ProgressStyle::with_template("✔️ Cleaning: {msg}").unwrap());

        self.progress.finish_with_message(format!(
            "Cleaned {} directories and {} files in {:?}.",
            self.result.directories,
            self.result.files,
            start.elapsed()
        ));

        Ok(())
    }

    fn clean_bin_obj(
        path: PathBuf,
        result: CleanerResult,
        progress: ProgressBar,
    ) -> Pin<Box<dyn Future<Output = CleanerResult>>> {
        Box::pin(async move {
            let mut result = result;
            let Ok(mut entries) = tokio::fs::read_dir(&path).await else {
                progress.println(format!(
                    "{}",
                    style(format!("❌ Unable to read {:?}", path)).red().bold()
                ));
                return result;
            };
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if !path.is_dir() || path.ends_with("packages") {
                    continue;
                }
                if path.ends_with("bin") || path.ends_with("obj") {
                    progress.set_message(format!("bin/obj ({:?})", path));
                    result.files +=
                        Self::delete_files(path.clone(), Vec::new(), progress.clone()).await;
                    if let Ok(()) = tokio::fs::remove_dir(path).await {
                        result.directories += 1;
                    }
                    continue;
                }
                result = Self::clean_bin_obj(path, result, progress.clone()).await;
            }

            result
        })
    }

    async fn clean_nuget(patterns: Vec<String>, progress: ProgressBar) -> CleanerResult {
        let paths = vec![
            PathBuf::from(".").join("packages"),
            PathBuf::from(std::env::var("USERPROFILE").unwrap())
                .join(".nuget")
                .join("packages"),
            PathBuf::from(std::env::var("LOCALAPPDATA").unwrap())
                .join("NuGet")
                .join("Cache"),
            PathBuf::from(std::env::var("LOCALAPPDATA").unwrap())
                .join("NuGet")
                .join("v3-cache"),
            PathBuf::from(std::env::var("LOCALAPPDATA").unwrap())
                .join("NuGet")
                .join("plugins-cache"),
            PathBuf::from(std::env::var("COMMONPROGRAMFILES(X86)").unwrap())
                .join("Microsoft SDKs")
                .join("NuGetPackages"),
        ];

        let mut result = CleanerResult::default();
        for path in paths {
            progress.set_message(format!("NuGet ({:?})", path));
            result.files +=
                Self::delete_files(path.clone(), patterns.clone(), progress.clone()).await;
            if let Ok(()) = tokio::fs::remove_dir(path).await {
                result.directories += 1;
            }
        }

        result
    }

    fn delete_files(
        path: PathBuf,
        patterns: Vec<String>,
        progress: ProgressBar,
    ) -> Pin<Box<dyn Future<Output = usize>>> {
        Box::pin(async move {
            let mut files = 0;
            let Ok(mut entries) = tokio::fs::read_dir(&path).await else {
                progress.println(format!(
                    "{}",
                    style(format!("❌ Unable to read {:?}", &path)).red().bold()
                ));
                return files;
            };
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                let has_pattern = patterns.is_empty()
                    || path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| patterns.iter().any(|p| n.contains(p)))
                        .unwrap_or_default();
                if path.is_dir() {
                    if !has_pattern {
                        continue;
                    }
                    files +=
                        Self::delete_files(path.clone(), patterns.clone(), progress.clone()).await;
                    _ = tokio::fs::remove_dir(path).await;
                    continue;
                }

                if !has_pattern && path.extension().and_then(|e| e.to_str()) == Some("dat") {
                    continue;
                }

                match tokio::fs::remove_file(&path).await {
                    Ok(_) => {
                        if let Ok(mut file) = tokio::fs::OpenOptions::new()
                            .write(true)
                            .append(true)
                            .create(true)
                            .open("C:\\temp\\cleaner.log")
                            .await
                        {
                            if files == 0 {
                                _ = file.write(b"-------------------------\n").await;
                            }
                            _ = file
                                .write(format!("removed: {}\n", path.display()).as_bytes())
                                .await;
                        }
                        files += 1;
                    }
                    Err(e) => progress.println(format!(
                        "{}: {}",
                        style(format!("❌ Unable to delete {:?}", path))
                            .red()
                            .bold(),
                        e
                    )),
                }
            }

            files
        })
    }
}
