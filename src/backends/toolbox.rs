use std::collections::{BTreeMap, BTreeSet};

use color_eyre::Result;
use color_eyre::eyre::eyre;
use serde::{Deserialize, Serialize};

use crate::cmd::{run_command, run_command_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Toolbox;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ToolboxConfig {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolboxPackageOptions {}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolboxRepoOptions {}

impl Backend for Toolbox {
    type Config = ToolboxConfig;
    type PackageOptions = ToolboxPackageOptions;
    type RepoOptions = ToolboxRepoOptions;

    fn invalid_package_help_text() -> String {
        String::new()
    }

    fn is_valid_package_name(_: &str) -> Option<bool> {
        None
    }

    fn get_all_packages(_: &Self::Config) -> Result<BTreeSet<String>> {
        Err(eyre!("unimplemented"))
    }

    fn get_installed_packages(
        config: &Self::Config,
    ) -> Result<BTreeMap<String, Self::PackageOptions>> {
        if Self::version(config).is_err() {
            return Ok(BTreeMap::new());
        }

        let output =
            run_command_for_stdout(["toolbox", "list", "--installed"], Perms::Same, StdErr::Show)?;

        Ok(parse_installed(&output)
            .into_iter()
            .map(|name| (name, Self::PackageOptions {}))
            .collect())
    }

    fn install_packages(
        packages: &BTreeMap<String, Self::PackageOptions>,
        _: bool,
        config: &Self::Config,
    ) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        if !toolbox_available(config) {
            log::warn!("toolbox is not installed; skipping toolbox package install");
            return Ok(());
        }
        run_command(
            ["toolbox", "install"]
                .into_iter()
                .chain(packages.keys().map(String::as_str)),
            Perms::Same,
        )
    }

    fn uninstall_packages(packages: &BTreeSet<String>, _: bool, config: &Self::Config) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        if !toolbox_available(config) {
            log::warn!("toolbox is not installed; skipping toolbox package uninstall");
            return Ok(());
        }
        for package in packages {
            run_command(["toolbox", "uninstall", package.as_str()], Perms::Same)?;
        }

        Ok(())
    }

    fn update_packages(packages: &BTreeSet<String>, _: bool, config: &Self::Config) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        if !toolbox_available(config) {
            log::warn!("toolbox is not installed; skipping toolbox package update");
            return Ok(());
        }
        for package in packages {
            run_command(["toolbox", "update", package.as_str()], Perms::Same)?;
        }

        Ok(())
    }

    fn update_all_packages(_: bool, config: &Self::Config) -> Result<()> {
        if !toolbox_available(config) {
            log::warn!("toolbox is not installed; skipping toolbox update");
            return Ok(());
        }
        run_command(["toolbox", "update"], Perms::Same)
    }

    fn clean_cache(_: &Self::Config) -> Result<()> {
        Ok(())
    }

    fn get_installed_repos(_: &Self::Config) -> Result<BTreeMap<String, Self::RepoOptions>> {
        Ok(BTreeMap::new())
    }

    fn add_repos(
        repos: &BTreeMap<String, Self::RepoOptions>,
        _: bool,
        _: &Self::Config,
    ) -> Result<()> {
        if repos.is_empty() {
            Ok(())
        } else {
            Err(eyre!("unimplemented"))
        }
    }

    fn remove_repos(repos: &BTreeSet<String>, _: bool, _: &Self::Config) -> Result<()> {
        if repos.is_empty() {
            Ok(())
        } else {
            Err(eyre!("unimplemented"))
        }
    }

    fn version(_: &Self::Config) -> Result<String> {
        run_command_for_stdout(["toolbox", "--version"], Perms::Same, StdErr::Show)
    }
}

fn toolbox_available(config: &ToolboxConfig) -> bool {
    Toolbox::version(config).is_ok()
}

fn parse_installed(output: &str) -> BTreeSet<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with("You can check for updates"))
        .filter_map(|line| line.split_whitespace().next())
        .filter(|token| !token.starts_with("----"))
        .filter(|token| *token != "Tool")
        .map(ToString::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_installed_table() {
        let output = "\
Tool                       Current Version   Released (UTC)
----                       ---------------   --------------
ada                        1.0.202218.0      2026-05-27 15:19
cr                         1.0.216341.0      2026-06-10 01:05

You can check for updates with `toolbox update`.
";

        let parsed = parse_installed(output);

        assert_eq!(
            parsed,
            ["ada".to_string(), "cr".to_string()].into_iter().collect()
        );
    }
}
