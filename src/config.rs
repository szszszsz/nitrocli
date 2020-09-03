// config.rs

// Copyright (C) 2020 The Nitrocli Developers
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::path;

use crate::args;

use anyhow::Context as _;

/// The name of nitrocli's configuration file relative to the
/// application configuration directory.
///
/// The application configuration directory is determined using the
/// `directories` crate.  For Unix, it is `$XDG_CONFIG_HOME/nitrocli`
/// (defaults to `$HOME/.config/nitrocli`).
const CONFIG_FILE: &str = "config.toml";

/// The configuration for nitrocli, usually read from configuration
/// files and environment variables.
#[derive(Clone, Copy, Debug, Default, PartialEq, merge::Merge, serde::Deserialize)]
pub struct Config {
  /// The model to connect to.
  pub model: Option<args::DeviceModel>,
  /// Whether to bypass the cache for all secrets or not.
  #[merge(strategy = merge::bool::overwrite_false)]
  #[serde(default)]
  pub no_cache: bool,
  /// The log level.
  #[merge(strategy = merge::num::overwrite_zero)]
  #[serde(default)]
  pub verbosity: u8,
}

impl Config {
  pub fn load() -> anyhow::Result<Self> {
    use merge::Merge as _;

    let mut config = Config::default();
    if let Some(user_config) = load_user_config()? {
      config.merge(user_config);
    }
    config.merge(load_env_config()?);

    Ok(config)
  }

  pub fn update(&mut self, args: &args::Args) {
    if args.model.is_some() {
      self.model = args.model;
    }
    if args.no_cache {
      self.no_cache = true;
    }
    if args.verbose > 0 {
      self.verbosity = args.verbose;
    }
  }
}

fn load_user_config() -> anyhow::Result<Option<Config>> {
  let project_dirs = directories::ProjectDirs::from("", "", "nitrocli")
    .ok_or_else(|| anyhow::anyhow!("Could not determine the nitrocli application directory"))?;
  let path = project_dirs.config_dir().join(CONFIG_FILE);
  if path.is_file() {
    read_config_file(&path).map(Some)
  } else {
    Ok(None)
  }
}

fn load_env_config() -> anyhow::Result<Config> {
  envy::prefixed("NITROCLI_")
    .from_env()
    .context("Failed to parse environment variables")
}

pub fn read_config_file(path: &path::Path) -> anyhow::Result<Config> {
  let s = fs::read_to_string(path)
    .with_context(|| format!("Failed to read configuration file '{}'", path.display()))?;
  toml::from_str(&s)
    .with_context(|| format!("Failed to parse configuration file '{}'", path.display()))
}
