use std::{future::Future, path::PathBuf, pin::Pin};

use anyhow::Error;

pub struct Cleaner {
    pub nuget: bool,
}

#[derive(Default, Clone, Copy)]
pub struct CleanerResult {
    pub directories: usize,
    pub files: usize,
}

impl std::ops::AddAssign for CleanerResult {
    fn add_assign(&mut self, rhs: Self) {
        self.directories += rhs.directories;
        self.files += rhs.files;
    }
}

impl Cleaner {
    pub async fn clean(&self) -> Result<CleanerResult, Error> {
        let mut result = Self::clean_bin_obj(PathBuf::from("."), CleanerResult::default()).await;
        if self.nuget {
            result += Self::clean_nuget().await;
        }

        Ok(result)
    }

    fn clean_bin_obj(
        path: PathBuf,
        result: CleanerResult,
    ) -> Pin<Box<dyn Future<Output = CleanerResult>>> {
        Box::pin(async move {
            let mut result = result;
            let Ok(mut entries) = tokio::fs::read_dir(&path).await else {
                eprintln!("Unable to read {:?}", path);
                return result;
            };
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                if path.ends_with("bin") || path.ends_with("obj") {
                    result.files += Self::delete_files(path.clone()).await;
                    if let Ok(()) = tokio::fs::remove_dir(path).await {
                        result.directories += 1;
                    }
                    continue;
                }
                result = Self::clean_bin_obj(path, result).await;
            }

            result
        })
    }

    async fn clean_nuget() -> CleanerResult {
        let paths = vec![
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
            result.files += Self::delete_files(path.clone()).await;
            if let Ok(()) = tokio::fs::remove_dir(path).await {
                result.directories += 1;
            }
        }

        result
    }

    fn delete_files(path: PathBuf) -> Pin<Box<dyn Future<Output = usize>>> {
        Box::pin(async move {
            let mut files = 0;
            let Ok(mut entries) = tokio::fs::read_dir(&path).await else {
                eprintln!("Unable to read {:?}", &path);
                return files;
            };
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.is_dir() {
                    files += Self::delete_files(path.clone()).await;
                    _ = tokio::fs::remove_dir(path).await;
                    continue;
                }

                match tokio::fs::remove_file(&path).await {
                    Ok(_) => files += 1,
                    Err(e) => eprintln!("Unable to delete {:?}: {}", path, e),
                }
            }

            files
        })
    }
}
