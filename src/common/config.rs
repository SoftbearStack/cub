// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: LGPL-3.0-or-later

use super::Error;
use serde::de::DeserializeOwned;
#[allow(deprecated)]
use std::env::home_dir;
use std::fs::read_to_string;

/// Configuration parameters for various features.
#[derive(Debug)]
pub struct CubConfig {
    debug_enabled: bool,
    toml: String,
}

impl CubConfig {
    /// Creates a configuration builder.
    pub fn builder() -> CubConfigBuilder {
        CubConfigBuilder {
            cub_config: None,
            debug_enabled: false,
            error: None,
        }
    }

    /// Returns `true` if debug is enabled.
    pub fn debug(&self) -> bool {
        self.debug_enabled
    }

    /// Returns configuration parameters.
    pub fn get<T: DeserializeOwned>(&self) -> Result<T, Error> {
        toml::from_str(&self.toml).map_err(|e: toml::de::Error| Error::String(format!("toml: {e}")))
    }
}

pub struct CubConfigBuilder {
    cub_config: Option<CubConfig>,
    debug_enabled: bool,
    error: Option<Error>,
}

impl CubConfigBuilder {
    pub fn build(self) -> Result<CubConfig, Error> {
        if let Some(error) = self.error {
            Err(error)
        } else if let Some(cub_config) = self.cub_config {
            Ok(cub_config)
        } else {
            Err(Error::String("config not set".to_string()))
        }
    }

    pub fn debug(self, debug_enabled: bool) -> Self {
        Self {
            cub_config: self.cub_config,
            debug_enabled,
            error: self.error,
        }
    }

    pub fn toml_file(self, file_name: &str) -> Self {
        let debug_enabled = self.debug_enabled;
        #[allow(deprecated)]
        let home_path = home_dir().and_then(|pathbuf| {
            if let Some(path) = pathbuf.to_str() {
                Some(format!("{path}/{file_name}"))
            } else {
                None
            }
        });
        let local_path = format!("./{file_name}");
        if let Some(cub_config) = home_path.and_then(|path| {
            read_to_string(path)
                .map(|s| CubConfig {
                    toml: s,
                    debug_enabled,
                })
                .ok()
        }) {
            Self {
                cub_config: Some(cub_config),
                debug_enabled,
                error: None,
            }
        } else {
            match read_to_string(&local_path)
                .map(|s| CubConfig {
                    toml: s,
                    debug_enabled,
                })
                .map_err(|_| Error::String(format!("{local_path}: cannot read")))
            {
                Ok(cfg) => Self {
                    cub_config: Some(cfg),
                    debug_enabled,
                    error: None,
                },
                Err(e) => Self {
                    cub_config: None,
                    debug_enabled,
                    error: Some(e),
                },
            }
        }
    }

    pub fn toml_str(self, toml: &str) -> Self {
        Self {
            cub_config: Some(CubConfig {
                toml: toml.to_string(),
                debug_enabled: self.debug_enabled,
            }),
            debug_enabled: self.debug_enabled,
            error: None,
        }
    }

    pub fn toml_string(self, toml: String) -> Self {
        Self {
            cub_config: Some(CubConfig {
                debug_enabled: self.debug_enabled,
                toml,
            }),
            debug_enabled: self.debug_enabled,
            error: None,
        }
    }
}
