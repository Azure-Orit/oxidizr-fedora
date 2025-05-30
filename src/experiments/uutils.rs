use crate::utils::Worker;
use anyhow::Result;
use std::path::{Path, PathBuf};
use tracing::info;

/// An experiment to install and configure a Rust-based replacement for a system utility.
pub struct UutilsExperiment<'a> {
    name: String,
    system: &'a dyn Worker,
    package: String,
    supported_releases: Vec<String>,
    unified_binary: Option<PathBuf>,
    bin_directory: PathBuf,
}

impl<'a> UutilsExperiment<'a> {
    /// Create a new UutilsExperiment.
    pub fn new(
        name: &str,
        system: &'a dyn Worker,
        package: &str,
        supported_releases: &[&str],
        unified_binary: Option<PathBuf>,
        bin_directory: PathBuf,
    ) -> Self {
        Self {
            name: name.to_string(),
            system,
            package: package.to_string(),
            supported_releases: supported_releases
                .iter()
                .map(|&release| release.to_string())
                .collect(),
            unified_binary,
            bin_directory,
        }
    }

    /// Check if the system is compatible with the experiment.
    pub fn check_compatible(&self) -> bool {
        self.supported_releases().contains(
            &self
                .system
                .distribution()
                .expect("unable to determine distribution information")
                .release,
        )
    }

    /// Reports the first supported release for the experiment.
    pub fn supported_releases(&self) -> Vec<String> {
        self.supported_releases.clone()
    }

    /// Check if the package is installed.
    pub fn check_installed(&self) -> bool {
        self.system.check_installed(&self.package).unwrap_or(false)
    }

    /// Report the name of the experiment.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Enable the experiment by installing and configuring the package.
    pub fn enable(&self) -> Result<()> {
        info!("Installing and configuring {}", self.package);
        self.system.install_package(&self.package)?;

        let files = self.system.list_files(self.bin_directory.clone())?;

        for f in files {
            let filename = f.file_name().unwrap().to_str().unwrap();
            let existing = match self.system.which(filename) {
                Ok(path) => path,
                Err(_) => Path::new("/usr/bin").join(filename),
            };

            if let Some(unified_binary) = &self.unified_binary {
                self.system
                    .replace_file_with_symlink(unified_binary.to_path_buf(), existing.clone())?;
            } else {
                self.system.replace_file_with_symlink(f, existing)?;
            }
        }

        Ok(())
    }

    /// Disable the experiment by removing the package and restoring the original files.
    pub fn disable(&self) -> Result<()> {
        let files = self.system.list_files(self.bin_directory.clone())?;

        for f in files {
            let filename = f.file_name().unwrap().to_str().unwrap();
            let existing = match self.system.which(filename) {
                Ok(path) => path,
                Err(_) => Path::new("/usr/bin").join(filename),
            };
            self.system.restore_file(existing)?;
        }

        info!("Removing {}", self.package);
        self.system.remove_package(&self.package)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{Distribution, MockSystem, vecs_eq};

    #[test]
    fn test_uutils_incompatible_distribution() {
        let runner = incompatible_runner();
        let coreutils = coreutils_fixture(&runner);
        assert!(!coreutils.check_compatible());
    }

    #[test]
    fn test_uutils_install_success_unified_binary() {
        let runner = coreutils_compatible_runner();
        let coreutils = coreutils_fixture(&runner);

        assert!(coreutils.enable().is_ok());

        let commands = runner.commands.clone().into_inner();
        assert_eq!(commands, &["dnf install -y uutils-coreutils"]);

        let backed_up_files = runner.backed_up_files.clone().into_inner();
        let expected = vec!["/usr/bin/date".to_string(), "/usr/bin/sort".to_string()];
        assert!(vecs_eq(backed_up_files, expected));

        let created_symlinks = runner.created_symlinks.clone().into_inner();
        let expected = vec![
            (
                "/usr/bin/coreutils".to_string(),
                "/usr/bin/sort".to_string(),
            ),
            (
                "/usr/bin/coreutils".to_string(),
                "/usr/bin/date".to_string(),
            ),
        ];

        assert!(vecs_eq(created_symlinks, expected));
        assert_eq!(runner.restored_files.clone().into_inner().len(), 0);
    }

    #[test]
    fn test_uutils_restore_installed() {
        let runner = coreutils_compatible_runner();
        runner.mock_install_package("uutils-coreutils");

        let coreutils = coreutils_fixture(&runner);
        assert!(coreutils.disable().is_ok());

        assert_eq!(runner.created_symlinks.clone().into_inner().len(), 0);
        assert_eq!(runner.backed_up_files.clone().into_inner().len(), 0);

        let commands = runner.commands.clone().into_inner();
        assert_eq!(commands.len(), 1);
        assert!(commands.contains(&"dnf remove -y uutils-coreutils".to_string()));

        let restored_files = runner.restored_files.clone().into_inner();
        let expected = vec!["/usr/bin/date".to_string(), "/usr/bin/sort".to_string()];
        assert!(vecs_eq(restored_files, expected));
    }

    fn coreutils_fixture(system: &MockSystem) -> UutilsExperiment {
        UutilsExperiment::new(
            "coreutils",
            system,
            "uutils-coreutils",
            &["42", "24.10", "25.04"],
            Some(PathBuf::from("/usr/bin/coreutils")),
            PathBuf::from("/usr/libexec/uutils-coreutils"),
        )
    }

    fn coreutils_compatible_runner() -> MockSystem {
        let runner = MockSystem::default();
        runner.mock_files(vec![
            ("/usr/libexec/uutils-coreutils/date", "", false),
            ("/usr/libexec/uutils-oreutils/sort", "", false),
            ("/usr/bin/sort", "", true),
            ("/usr/bin/date", "", true),
        ]);
        runner
    }

    fn incompatible_runner() -> MockSystem {
        MockSystem::new(Distribution {
            id: "Fedora".to_string(),
            release: "41".to_string(),
        })
    }
}
