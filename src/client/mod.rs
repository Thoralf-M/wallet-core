// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

pub mod api;
// pub(crate) mod mqtt;
pub mod node;
pub mod options;

use getset::Getters;
use iota_client::{node_manager::validate_url, Client, ClientBuilder};
use once_cell::sync::Lazy;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use tokio::sync::{Mutex, RwLock};
use url::Url;

use crate::{
    client::options::{ClientOptions}
};

use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

type ClientInstanceMap = Arc<Mutex<HashMap<ClientOptions, Arc<RwLock<Client>>>>>;

/// Gets the client instances map.
fn instances() -> &'static ClientInstanceMap {
    static INSTANCES: Lazy<ClientInstanceMap> = Lazy::new(Default::default);
    &INSTANCES
}

pub(crate) async fn get_client(options: &ClientOptions) -> crate::Result<Arc<RwLock<Client>>> {
    let mut map = instances().lock().await;

    if !map.contains_key(options) {
        let mut client_builder = ClientBuilder::new()
            // .with_mqtt_broker_options(
            //     options
            //         .mqtt_broker_options()
            //         .as_ref()
            //         .map(|options| options.clone().into())
            //         .unwrap_or_else(|| {
            //             iota_client::BrokerOptions::new().automatic_disconnect(false)
            //         }),
            // )
            .with_local_pow(*options.local_pow())
            .with_node_pool_urls(
                &options
                    .node_pool_urls()
                    .iter()
                    .map(|url| url.to_string())
                    .collect::<Vec<String>>()[..],
            )
            .await
            // safe to unwrap since we're sure we have valid URLs
            .unwrap();

        if let Some(network) = options.network() {
            client_builder = client_builder.with_network(network);
        }

        for node in options.nodes() {
            if !node.disabled {
                if let Some(auth) = &node.auth {
                    client_builder = client_builder.with_node_auth(
                        node.url.as_str(),
                        auth.jwt.clone(),
                        auth.basic_auth_name_pwd.as_ref().map(|(ref x, ref y)| (&x[..], &y[..])),
                    )?;
                } else {
                    // safe to unwrap since we're sure we have valid URLs
                    client_builder = client_builder.with_node(node.url.as_str()).unwrap();
                }
            }
        }

        if let Some(primary_node) = options.primary_node() {
            if !primary_node.disabled {
                if let Some(auth) = &primary_node.auth {
                    client_builder = client_builder.with_primary_node(
                        primary_node.url.as_str(),
                        auth.jwt.clone(),
                        auth.basic_auth_name_pwd.as_ref().map(|(ref x, ref y)| (&x[..], &y[..])),
                    )?;
                } else {
                    // safe to unwrap since we're sure we have valid URLs
                    client_builder = client_builder
                        .with_primary_node(primary_node.url.as_str(), None, None)
                        .unwrap();
                }
            }
        }

        if let Some(primary_pow_node) = options.primary_pow_node() {
            if !primary_pow_node.disabled {
                if let Some(auth) = &primary_pow_node.auth {
                    client_builder = client_builder.with_primary_pow_node(
                        primary_pow_node.url.as_str(),
                        auth.jwt.clone(),
                        auth.basic_auth_name_pwd.as_ref().map(|(ref x, ref y)| (&x[..], &y[..])),
                    )?;
                } else {
                    // safe to unwrap since we're sure we have valid URLs
                    client_builder = client_builder
                        .with_primary_pow_node(primary_pow_node.url.as_str(), None, None)
                        .unwrap();
                }
            }
        }

        if let Some(node_sync_interval) = options.node_sync_interval() {
            client_builder = client_builder.with_node_sync_interval(*node_sync_interval);
        }

        if !options.node_sync_enabled() {
            client_builder = client_builder.with_node_sync_disabled();
        }

        if let Some(request_timeout) = options.request_timeout() {
            client_builder = client_builder.with_request_timeout(*request_timeout);
        }

        for (api, timeout) in options.api_timeout() {
            client_builder = client_builder.with_api_timeout(api.clone().into(), *timeout);
        }

        let client = client_builder.finish().await?;

        map.insert(options.clone(), Arc::new(RwLock::new(client)));
    }

    // safe to unwrap since we make sure the client exists on the block above
    let client = map.get(options).unwrap();

    Ok(client.clone())
}

/// Drops all clients.
pub async fn drop_all() {
    instances().lock().await.clear();
}

fn convert_urls(urls: &[&str]) -> crate::Result<Vec<Url>> {
    let mut err = None;
    let urls: Vec<Option<Url>> = urls
        .iter()
        .map(|node| match Url::parse(node) {
            Ok(url) => match validate_url(url) {
                Ok(url) => Some(url),
                Err(e) => {
                    err.replace(e);
                    None
                }
            },
            Err(e) => {
                err.replace(e.into());
                None
            }
        })
        .collect();

    if let Some(err) = err {
        Err(err.into())
    } else {
        // safe to unwrap: all URLs were parsed above
        let urls = urls.iter().map(|url| url.clone().unwrap()).collect();
        Ok(urls)
    }
}
