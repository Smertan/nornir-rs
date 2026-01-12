use crate::CustomTreeMap;
use nornir_core_derive::{DerefMacro, DerefMutMacro};
use schemars::{schema_for, JsonSchema};
use serde::de::{Error, SeqAccess, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use std::sync::Arc;

pub trait BaseMethods {
    fn schema() -> String
    where
        Self: Sized,
        Self: JsonSchema,
    {
        let schema = schema_for!(Self);
        serde_json::to_string_pretty(&schema).unwrap()
    }
}

pub trait BaseBuilderHost {
    type Output;

    // Updates the hostname and returns the updated builder.
    fn hostname(self, hostname: &str) -> Self;

    /// Updates the port and returns the updated builder.
    fn port(self, port: u16) -> Self;

    /// Updates the username and returns the updated builder.
    fn username(self, username: &str) -> Self;

    /// Updates the password and returns the updated builder.
    fn password(self, password: &str) -> Self;

    /// Updates the platform and returns the updated builder.
    fn platform(self, platform: &str) -> Self;

    /// Updates the groups and returns the updated builder.
    fn groups(self, groups: ParentGroups) -> Self;

    /// Updates the data and returns the updated builder.
    fn data(self, data: Data) -> Self;

    /// Updates the connection options and returns the updated builder.
    fn connection_options(self, name: String, options: ConnectionOptions) -> Self;

    /// Updates the defaults and returns the updated builder.
    fn defaults(self, defaults: &Arc<Defaults>) -> Self;

    /// Builds the struct from the updated builder and returns final struct object.
    fn build(self) -> Self::Output;
}

// Required for the DerefMacro derive to satisfy the DerefTarget trait.
pub trait DerefTarget {
    type Target;
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ConnectionOptions {
    pub hostname: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub platform: Option<String>,
    pub extras: Option<Extras>,
}

impl Default for ConnectionOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionOptions {
    pub fn new() -> Self {
        ConnectionOptions {
            hostname: None,
            port: None,
            username: None,
            password: None,
            platform: None,
            extras: None,
        }
    }
}

impl DerefTarget for Extras {
    type Target = serde_json::Value;
}

/// The DataExtra struct is a wrapper for serde_json::Value, any json data is accepted.
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema, DerefMacro, DerefMutMacro,
)]
pub struct Extras(serde_json::Value);

impl DerefTarget for ParentGroups {
    type Target = Vec<String>;
}

/// The ParentGroups struct is a wrapped vector of strings.
///
/// It stores a list of strings representing the groups the host
/// belongs to.
///
/// The ParentGroups struct implements Deref and DerefMut for easy
/// access to the underlying vector.
#[derive(Debug, Clone, Serialize, PartialEq, JsonSchema, DerefMacro, DerefMutMacro)]
pub struct ParentGroups(Vec<String>);

impl Default for ParentGroups {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentGroups {
    pub fn new() -> Self {
        ParentGroups(Vec::new())
    }
}

impl<'de> Deserialize<'de> for ParentGroups {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match deserializer.deserialize_seq(ParentGroupsVisitor) {
            Ok(parent) => Ok(parent),
            Err(err) => {
                log::error!("{}", err);
                let err_msg = "Groups should be an array of strings for use with `ParentGroups`";
                log::error!("{err_msg}");
                Err(D::Error::custom(err_msg))
            }
        }
    }
}

struct ParentGroupsVisitor;

impl<'de> Visitor<'de> for ParentGroupsVisitor {
    type Value = ParentGroups;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a sequence of strings")
    }
    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Err(Error::invalid_value(Unexpected::Str(s), &self))
    }

    /// This method is used to handle custom deserialization logic for
    /// sequences. It returns a list of unique strings from the sequence.
    ///
    /// The vector implementation ensures that duplicate strings are not added to the
    /// and preserves the order of the first occurrence of each string.
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut groups = Vec::new();
        while let Some(value) = seq.next_element()? {
            if !groups.contains(&value) {
                groups.push(value);
            }
        }

        Ok(ParentGroups(groups.into_iter().collect()))
    }
}

impl DerefTarget for Defaults {
    type Target = serde_json::Value;
}

#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, DerefMacro, DerefMutMacro,
)]
pub struct Defaults(serde_json::Value);

impl DerefTarget for Data {
    type Target = serde_json::Value;
}

/// The Data struct is a wrapper for serde_json::Value, any json data is accepted.
#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, DerefMacro, DerefMutMacro,
)]
pub struct Data(serde_json::Value);

impl Data {
    pub fn new(data: serde_json::Value) -> Self {
        Data(data)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Host {
    pub name: String,
    pub hostname: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub platform: Option<String>,
    pub groups: Option<ParentGroups>,
    pub data: Option<Data>,
    pub connection_options: Option<CustomTreeMap<ConnectionOptions>>,
    pub defaults: Option<Arc<Defaults>>,
}

impl Host {
    pub fn new(name: &str) -> Host {
        Host {
            name: name.to_string(),
            hostname: None,
            port: None,
            username: None,
            password: None,
            platform: None,
            groups: None,
            data: None,
            connection_options: None,
            defaults: None,
        }
    }
    pub fn builder(name: &str) -> HostBuilder {
        HostBuilder::new(name)
    }
}

impl BaseMethods for Host {}

pub struct HostBuilder {
    name: String,
    hostname: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    platform: Option<String>,
    groups: Option<ParentGroups>,
    data: Option<Data>,
    connection_options: Option<CustomTreeMap<ConnectionOptions>>,
    defaults: Option<Arc<Defaults>>,
}

impl HostBuilder {
    pub fn new(name: &str) -> Self {
        HostBuilder {
            name: name.to_string(),
            hostname: None,
            port: None,
            username: None,
            password: None,
            platform: None,
            groups: None,
            data: None,
            connection_options: None,
            defaults: None,
        }
    }
}

impl BaseBuilderHost for HostBuilder {
    type Output = Host;

    fn hostname(mut self, hostname: &str) -> Self {
        self.hostname = Some(hostname.to_string());
        self
    }

    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    fn username(mut self, username: &str) -> Self {
        self.username = Some(username.to_string());
        self
    }

    fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self
    }

    fn platform(mut self, platform: &str) -> Self {
        self.platform = Some(platform.to_string());
        self
    }

    fn groups(mut self, groups: ParentGroups) -> Self {
        self.groups = Some(groups);
        self
    }

    fn data(mut self, data: Data) -> Self {
        self.data = Some(data);
        self
    }

    fn connection_options(mut self, name: String, options: ConnectionOptions) -> Self {
        if self.connection_options.is_none() {
            self.connection_options = Some(CustomTreeMap::new());
        }
        self.connection_options
            .as_mut()
            .unwrap()
            .insert(name, options);
        self
    }

    fn defaults(mut self, defaults: &Arc<Defaults>) -> Self {
        self.defaults = Some(Arc::clone(defaults));
        self
    }

    fn build(self) -> Host {
        Host {
            name: self.name,
            hostname: self.hostname,
            port: self.port,
            username: self.username,
            password: self.password,
            platform: self.platform,
            groups: self.groups,
            data: self.data,
            connection_options: self.connection_options,
            defaults: self.defaults,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Group {
    pub hostname: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub platform: Option<String>,
    pub groups: Option<ParentGroups>,
    pub data: Option<Data>,
    pub connection_options: Option<CustomTreeMap<ConnectionOptions>>,
    pub defaults: Option<Arc<Defaults>>,
}

impl Default for Group {
    fn default() -> Group {
        Group::new()
    }
}

impl Group {
    pub fn new() -> Group {
        Group {
            hostname: None,
            port: None,
            username: None,
            password: None,
            platform: None,
            groups: None,
            data: None,
            connection_options: None,
            defaults: None,
        }
    }
    pub fn builder(hostname: &str) -> GroupBuilder {
        GroupBuilder::new(hostname)
    }
}

pub struct GroupBuilder {
    pub hostname: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub platform: Option<String>,
    pub groups: Option<ParentGroups>,
    pub data: Option<Data>,
    pub connection_options: Option<CustomTreeMap<ConnectionOptions>>,
    pub defaults: Option<Arc<Defaults>>,
}

impl BaseBuilderHost for GroupBuilder {
    type Output = Group;

    fn hostname(mut self, hostname: &str) -> Self {
        self.hostname = Some(hostname.to_string());
        self
    }
    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    fn username(mut self, username: &str) -> Self {
        self.username = Some(username.to_string());
        self
    }

    fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_string());
        self
    }
    fn platform(mut self, platform: &str) -> Self {
        self.platform = Some(platform.to_string());
        self
    }
    fn groups(mut self, groups: ParentGroups) -> Self {
        self.groups = Some(groups);
        self
    }
    fn data(mut self, data: Data) -> Self {
        self.data = Some(data);
        self
    }
    fn connection_options(mut self, name: String, options: ConnectionOptions) -> Self {
        if self.connection_options.is_none() {
            self.connection_options = Some(CustomTreeMap::new());
        }
        self.connection_options
            .as_mut()
            .unwrap()
            .insert(name, options);
        self
    }
    fn defaults(mut self, defaults: &Arc<Defaults>) -> Self {
        self.defaults = Some(Arc::clone(defaults));
        self
    }
    fn build(self) -> Group {
        Group {
            hostname: self.hostname,
            port: self.port,
            username: self.username,
            password: self.password,
            platform: self.platform,
            groups: self.groups,
            data: self.data,
            connection_options: self.connection_options,
            defaults: self.defaults,
        }
    }
}

impl GroupBuilder {
    pub fn new(hostname: &str) -> Self {
        GroupBuilder {
            hostname: Some(hostname.to_string()),
            port: None,
            username: None,
            password: None,
            platform: None,
            groups: None,
            data: None,
            connection_options: None,
            defaults: None,
        }
    }
}

pub type HostsTarget = CustomTreeMap<Host>;

impl DerefTarget for Hosts {
    type Target = CustomTreeMap<Host>;
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, DerefMacro, DerefMutMacro)]
#[serde(deny_unknown_fields)]
pub struct Hosts(HostsTarget);

impl Default for Hosts {
    fn default() -> Self {
        Self::new()
    }
}

impl Hosts {
    pub fn new() -> Self {
        Hosts(CustomTreeMap::new())
    }

    pub fn add_host(&mut self, host: Host) {
        let name = host.name.clone();
        self.insert(name, host);
    }
}

impl BaseMethods for Hosts {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, DerefMacro, DerefMutMacro)]
pub struct Groups(CustomTreeMap<Group>);

impl DerefTarget for Groups {
    type Target = CustomTreeMap<Group>;
}

/// The TransformFunctionOptions struct is a wrapper for serde_json::Value, any json data is accepted.
#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, DerefMacro, DerefMutMacro,
)]
pub struct TransformFunctionOptions(serde_json::Value);

impl DerefTarget for TransformFunctionOptions {
    type Target = serde_json::Value;
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Inventory {
    pub hosts: Hosts,
    pub groups: Option<Groups>,
    pub defaults: Option<Defaults>,
    // TODO: add transform_function
    // transform_function: Option<TransformFunction>,
    pub transform_function_options: Option<TransformFunctionOptions>,
}

impl BaseMethods for Inventory {}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            hosts: Hosts::new(),
            groups: None,
            defaults: None,
            transform_function_options: None,
        }
    }

    pub fn builder() -> InventoryBuilder {
        InventoryBuilder::new()
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Inventory::new()
    }
}
pub struct InventoryBuilder {
    pub hosts: Option<Hosts>,
    pub groups: Option<Groups>,
    pub defaults: Option<Defaults>,
    pub transform_function_options: Option<TransformFunctionOptions>,
}

impl InventoryBuilder {
    pub fn new() -> InventoryBuilder {
        InventoryBuilder {
            hosts: None,
            groups: None,
            defaults: None,
            transform_function_options: None,
        }
    }

    pub fn hosts(mut self, hosts: Hosts) -> Self {
        self.hosts = Some(hosts);
        self
    }

    pub fn groups(mut self, groups: Groups) -> Self {
        self.groups = Some(groups);
        self
    }

    pub fn defaults(mut self, defaults: Defaults) -> Self {
        self.defaults = Some(defaults);
        self
    }

    pub fn transform_function_options(mut self, options: TransformFunctionOptions) -> Self {
        self.transform_function_options = Some(options);
        self
    }

    pub fn build(self) -> Inventory {
        Inventory {
            hosts: self.hosts.unwrap_or_default(),
            groups: self.groups,
            defaults: self.defaults,
            transform_function_options: self.transform_function_options,
        }
    }
}

impl Default for InventoryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_dummy_hosts() -> Result<Hosts, std::io::Error> {
        let mut hosts = Hosts(CustomTreeMap::new());
        // hosts.insert("hosts".to_string(), CustomTreeMap::new());
        for i in 1..=10 {
            let mut groups = ParentGroups::new();
            groups.push("cisco".to_string());
            let host = Host::builder(&format!("host{}.example.com", i))
                .port(2200 + i as u16)
                .username(&format!("user{}", i))
                .password(&format!("password{}", i))
                .platform(if i % 2 == 0 { "linux" } else { "windows" })
                .data(Data(serde_json::json!(vec![format!(
                    "data for host {}",
                    i
                )])))
                .groups(groups)
                .connection_options(String::from("Cisco"), ConnectionOptions::new())
                .build();

            let hostname = host.name.clone();

            hosts.insert(hostname, host);
        }

        Ok(hosts)
    }

    #[test]
    fn test_host_new() {
        let host = Host::new("example.com");
        assert_eq!(host.hostname, None);
        assert_eq!(host.port, None);
        assert_eq!(host.username, None);
        assert_eq!(host.password, None);
        assert_eq!(host.platform, None);
        assert_eq!(host.groups, None);
        assert_eq!(host.data, None);
        assert_eq!(host.connection_options, None);
        assert_eq!(host.defaults.as_ref(), None);
    }

    #[test]
    fn test_hosts_new() {
        let mut hosts = Hosts::new();

        // Add 10 hosts to the hosts map with dummy data
        for i in 1..=10 {
            let host = Host::builder(&format!("host{}.example.com", i))
                .port(2200 + i as u16)
                .username(&format!("user{}", i))
                .password(&format!("password{}", i))
                .platform(if i % 2 == 0 { "linux" } else { "windows" })
                .data(Data(serde_json::json!(vec![format!(
                    "data for host {}",
                    i
                )])))
                .connection_options(String::from("Juniper"), ConnectionOptions::new())
                .build();

            hosts.add_host(host);
        }
        assert_eq!(hosts.len(), 10);
    }

    #[test]
    fn test_build_hosts() {
        let hosts = create_dummy_hosts();
        assert_eq!(hosts.unwrap().len(), 10);
    }

    #[test]
    fn test_parent_groups() {
        let groups = vec![
            "cisco".to_string(),
            "Juniper".to_string(),
            "arista".to_string(),
        ];
        let serialized = serde_json::to_string(&groups).unwrap();
        assert_eq!(serialized, "[\"cisco\",\"Juniper\",\"arista\"]");
        let mut deserialized: ParentGroups = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.sort(), ParentGroups(groups).sort());
    }

    #[test]
    fn test_parent_groups_deduplication() {
        // Test that duplicate groups are removed during deserialization
        let groups_with_duplicates = vec![
            "cisco".to_string(),
            "juniper".to_string(),
            "cisco".to_string(), // duplicate
            "arista".to_string(),
            "juniper".to_string(), // duplicate
            "cisco".to_string(),   // duplicate
        ];

        let serialized = serde_json::to_string(&groups_with_duplicates).unwrap();
        let deserialized: ParentGroups = serde_json::from_str(&serialized).unwrap();

        // Should only contain unique values in order of first occurrence
        assert_eq!(deserialized.len(), 3);
        assert_eq!(deserialized[0], "cisco");
        assert_eq!(deserialized[1], "juniper");
        assert_eq!(deserialized[2], "arista");
    }

    #[test]
    fn test_parent_groups_preserves_order() {
        // Test that the order of first occurrence is preserved
        let groups = vec![
            "zebra".to_string(),
            "apple".to_string(),
            "zebra".to_string(), // duplicate
            "banana".to_string(),
        ];

        let serialized = serde_json::to_string(&groups).unwrap();
        let deserialized: ParentGroups = serde_json::from_str(&serialized).unwrap();

        // Should preserve order of first occurrence
        assert_eq!(deserialized.len(), 3);
        assert_eq!(deserialized[0], "zebra");
        assert_eq!(deserialized[1], "apple");
        assert_eq!(deserialized[2], "banana");
    }

    /// Tests the ParentGroups deserialization with an error.
    ///
    /// The error message is expected to be "Groups should be an array of strings for use with `ParentGroups`"
    #[test]
    fn test_parent_groups_err() {
        let name = String::from("name");
        let deserialized: Result<ParentGroups, serde_json::Error> = serde_json::from_str(&name);
        match deserialized {
            Ok(_) => panic!("Expected an error, but deserialization succeeded"),
            Err(err) => {
                assert_eq!(
                    err.to_string(),
                    "Groups should be an array of strings for use with `ParentGroups`"
                );
            }
        }
    }

    // TODO: Create a test to verify the Host defaults deserialization
}
