use getset::Getters;
use iota_client::{node_manager::validate_url, Client, ClientBuilder};
use once_cell::sync::Lazy;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use tokio::sync::{Mutex, RwLock};
use url::Url;

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

/// Node authentication object.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeAuth {
    /// JWT.
    pub jwt: Option<String>,
    /// Username and password.
    pub basic_auth_name_pwd: Option<(String, String)>,
}

/// Node definition.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash, Getters)]
#[getset(get = "pub(crate)")]
pub struct Node {
    /// Node url.
    pub url: Url,
    /// Node auth options.
    pub auth: Option<NodeAuth>,
    /// Whether the node is disabled or not.
    #[serde(default)]
    pub disabled: bool,
}

impl From<Url> for Node {
    fn from(url: Url) -> Self {
        Self {
            url,
            auth: None,
            disabled: false,
        }
    }
}
