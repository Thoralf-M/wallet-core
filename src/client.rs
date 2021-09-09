/// The client options type.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, Getters)]
/// Need to set the get methods to be public for binding
#[getset(get = "pub")]
pub struct ClientOptions {
    /// The primary node to connect to.
    #[serde(rename = "node")] // here just for DB compatibility; can be changed when migrations are implemented
    primary_node: Option<Node>,
    /// The primary PoW node to connect to.
    #[serde(rename = "primaryPoWNode")]
    primary_pow_node: Option<Node>,
    /// The nodes to connect to.
    #[serde(default)]
    nodes: Vec<Node>,
    /// The node pool urls.
    #[serde(rename = "nodePoolUrls", default)]
    node_pool_urls: Vec<Url>,
    /// The network string.
    network: Option<String>,
    /// The MQTT broker options.
    #[serde(rename = "mqttBrokerOptions")]
    mqtt_broker_options: Option<BrokerOptions>,
    /// Enable local proof-of-work or not.
    #[serde(rename = "localPow", default = "default_local_pow")]
    local_pow: bool,
    /// The node sync interval.
    #[serde(rename = "nodeSyncInterval")]
    node_sync_interval: Option<Duration>,
    /// Enable node synchronization or not.
    #[serde(rename = "nodeSyncEnabled", default = "default_node_sync_enabled")]
    node_sync_enabled: bool,
    /// Enable mqtt or not.
    #[serde(rename = "mqttEnabled", default = "default_mqtt_enabled")]
    mqtt_enabled: bool,
    /// The request timeout.
    #[serde(rename = "requestTimeout")]
    request_timeout: Option<Duration>,
    /// The API timeout.
    #[serde(rename = "apiTimeout", default)]
    api_timeout: HashMap<Api, Duration>,
}