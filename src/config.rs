//! Structures representing configuration data.

use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::Utf8Error;

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use toml;
use url::Url;


/// The path to the configuration file.
pub(crate) static CONFIG_PATH: OnceCell<PathBuf> = OnceCell::new();

/// The current configuration of icingcake, behind an [RwLock].
pub(crate) static CONFIG: OnceCell<RwLock<Config>> = OnceCell::new();


/// icingcake's full configuration.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct Config {
    pub http_server: HttpServerConfig,
    pub icinga_api: IcingaApiConfig,
}

/// Configuration related to the HTTP server.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct HttpServerConfig {
    /// IP address and port on which to listen for connections.
    pub listen_socket_address: SocketAddr,
}

/// Configuration related to the Icinga API.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct IcingaApiConfig {
    /// Base URL of the Icinga API.
    pub base_url: Url,

    /// Username with which to authenticate against the Icinga API.
    pub username: String,

    /// Password with which to authenticate against the Icinga API.
    pub password: String,
}


/// An error that may occur when loading the configuration.
#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum ConfigLoadError {
    #[non_exhaustive] Opening { error: io::Error },
    #[non_exhaustive] Reading { error: io::Error },
    #[non_exhaustive] Decoding { error: Utf8Error },
    #[non_exhaustive] Parsing { error: toml::de::Error },
}
impl fmt::Display for ConfigLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Opening { error, .. }
                => write!(f, "error opening config file: {}", error),
            Self::Reading { error, .. }
                => write!(f, "error reading config file: {}", error),
            Self::Decoding { error, .. }
                => write!(f, "error decoding config file: {}", error),
            Self::Parsing { error, .. }
                => write!(f, "error parsing config file: {}", error),
        }
    }
}
impl std::error::Error for ConfigLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Opening { error, .. } => Some(error),
            Self::Reading { error, .. } => Some(error),
            Self::Decoding { error, .. } => Some(error),
            Self::Parsing { error, .. } => Some(error),
        }
    }
}


/// Loads the configuration.
pub(crate) fn load() -> Result<Config, ConfigLoadError> {
    let config_path = CONFIG_PATH.get().expect("CONFIG_PATH not set?!");

    let mut file = File::open(config_path)
        .map_err(|error| ConfigLoadError::Opening { error })?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|error| ConfigLoadError::Reading { error })?;
    let string = std::str::from_utf8(buf.as_slice())
        .map_err(|error| ConfigLoadError::Decoding { error })?;
    toml::from_str(&string)
        .map_err(|error| ConfigLoadError::Parsing { error })
}
