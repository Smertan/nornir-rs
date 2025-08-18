use schemars::{schema_for, JsonSchema};
use serde::de::{Error, SeqAccess, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize}; // , Serializer};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::ops::{Deref, DerefMut};


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
    fn data(self, data: Vec<String>) -> Self;

    /// Updates the connection options and returns the updated builder.
    fn connection_options(self, options: ConnectionOptions) -> Self;

    /// Updates the defaults and returns the updated builder.
    fn defaults(self, defaults: Defaults) -> Self;

    /// Builds the struct from the updated builder and returns final struct object.
    fn build(self) -> Self::Output;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ConnectionOptions {
    pub hostname: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub platform: Option<String>,
    pub extras: Option<String>,
}

impl ConnectionOptions {
    pub fn new(hostname: &str) -> Self {
        ConnectionOptions {
            hostname: hostname.to_string(),
            port: Some(22),
            username: None,
            password: None,
            platform: None,
            extras: None,
        }
    }
}

/// The ParentGroups struct is a wrapped vector of strings.
///
/// The ParentGroups struct implements Deref and DerefMut for easy access to the underlying vector.
#[derive(Debug, Clone, Serialize, PartialEq, JsonSchema)]
pub struct ParentGroups(Vec<String>);

impl ParentGroups {
    pub fn new() -> Self {
        ParentGroups(Vec::new())
    }
}

impl Deref for ParentGroups {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ParentGroups {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // println!("Parsing groups {:?}", seq.size_hint());

        let mut groups: HashSet<String> = HashSet::new();
        while let Some(value) = seq.next_element()? {
            // println!("{value}");
            groups.insert(value);
        }

        Ok(ParentGroups(groups.into_iter().collect()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Defaults(Option<serde_json::Value>);

impl Deref for Defaults {
    type Target = Option<serde_json::Value>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Defaults {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Host {
    pub name: String,
    pub hostname: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub platform: Option<String>,
    pub groups: Option<ParentGroups>,
    pub data: Option<Vec<String>>,
    pub connection_options: Option<ConnectionOptions>,
    // #[serde(flatten)]
    pub defaults: Defaults,
}

impl Host {
    pub fn new(name: &str, hostname: &str) -> Host {
        Host {
            name: name.to_string(),
            hostname: hostname.to_string(),
            port: Some(22),
            username: None,
            password: None,
            platform: None,
            groups: None,
            data: None,
            connection_options: None,
            // defaults: Defaults(Some(serde_json::json!({
            //     "platform": "linux"
            // }))),
            defaults: Defaults(None),
        }
    }
    pub fn builder(name: &str, hostname: &str) -> HostBuilder {
        HostBuilder::new(name, hostname)
    }
}

impl BaseMethods for Host {}


pub struct HostBuilder {
    name: String,
    hostname: String,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    platform: Option<String>,
    groups: Option<ParentGroups>,
    data: Option<Vec<String>>,
    connection_options: Option<ConnectionOptions>,
    defaults: Defaults,
}

impl HostBuilder {
    pub fn new(name: &str, hostname: &str) -> Self {
        HostBuilder {
            name: name.to_string(),
            hostname: hostname.to_string(),
            port: Some(22),
            username: None,
            password: None,
            platform: None,
            groups: None,
            data: None,
            connection_options: None,
            defaults: Defaults(Some(serde_json::json!({
                "platform": "linux"
            }))),
        }
    }
}

impl BaseBuilderHost for HostBuilder {
    type Output = Host;
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

    fn data(mut self, data: Vec<String>) -> Self {
        self.data = Some(data);
        self
    }

    fn connection_options(mut self, options: ConnectionOptions) -> Self {
        self.connection_options = Some(options);
        self
    }

    fn defaults(mut self, defaults: Defaults) -> Self {
        self.defaults = defaults;
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

pub struct Group {
    pub hostname: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub platform: Option<String>,
    pub groups: Option<ParentGroups>,
    pub data: Option<Vec<String>>,
    pub connection_options: Option<ConnectionOptions>,
    pub defaults: Defaults,
}

impl Group {
    pub fn new(hostname: &str) -> Group {
        Group {
            hostname: hostname.to_string(),
            port: Some(22),
            username: None,
            password: None,
            platform: None,
            groups: None,
            data: None,
            connection_options: None,
            defaults: Defaults(None),
        }
    }
    pub fn builder(hostname: &str) -> GroupBuilder {
        GroupBuilder::new(hostname)
    }
}

pub struct GroupBuilder {
    pub hostname: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub platform: Option<String>,
    pub groups: Option<ParentGroups>,
    pub data: Option<Vec<String>>,
    pub connection_options: Option<ConnectionOptions>,
    pub defaults: Defaults,
}

impl BaseBuilderHost for GroupBuilder {
    type Output = Group;
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
    fn data(mut self, data: Vec<String>) -> Self {
        self.data = Some(data);
        self
    }
    fn connection_options(mut self, options: ConnectionOptions) -> Self {
        self.connection_options = Some(options);
        self
    }
    fn defaults(mut self, defaults: Defaults) -> Self {
        self.defaults = defaults;
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
            hostname: hostname.to_string(),
            port: Some(22),
            username: None,
            password: None,
            platform: None,
            groups: None,
            data: None,
            connection_options: None,
            defaults: Defaults(None),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Hosts(HashMap<String, Host>);
// pub struct Hosts {
//     pub hosts: HashMap<String, Host>,
// }

impl Deref for Hosts {
    type Target = HashMap<String, Host>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Hosts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Hosts {
    pub fn new() -> Self {
        Hosts(HashMap::new())
    }

    // pub fn new() -> Hosts {
    //     Hosts {
    //         hosts: HashMap::new(),
    //     }
    // }
    pub fn add_host(&mut self, host: Host) {
        self.insert(host.hostname.clone(), host);
    }
}

pub fn create_dummy_hosts() -> Result<(), std::io::Error> {
    let mut hosts = HashMap::new();
    // hosts.insert("hosts".to_string(), HashMap::new());
    for i in 1..=10 {
        let mut groups = ParentGroups::new();
        groups.push("cisco".to_string());
        let host = Host::builder(
            &format!("host{}.example.com", i),
            &format!("host{}.example.com", i),
        )
        .port(2200 + i as u16)
        .username(&format!("user{}", i))
        .password(&format!("password{}", i))
        .platform(if i % 2 == 0 { "linux" } else { "windows" })
        .data(vec![format!("data for host {}", i)])
        .groups(groups)
        .connection_options(ConnectionOptions::new(&format!("host{}.example.com", i)))
        .build();

        let hostname = host.name.clone();

        // Tries to get the hosts object from the hosts map or creates an entry with an empty hashmap.
        hosts
            .entry("hosts".to_string())
            .or_insert_with(HashMap::new)
            .insert(hostname, host);
    }

    let json = serde_json::to_string_pretty(&hosts)?;
    let mut file = File::create("env/dummy_hosts.json")?;
    file.write_all(json.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_new() {
        let host = Host::new("example.com", "example.com");
        assert_eq!(host.hostname, "example.com");
        assert_eq!(host.port, Some(22));
        assert_eq!(host.username, None);
        assert_eq!(host.password, None);
        assert_eq!(host.platform, None);
        assert_eq!(host.groups, None);
        assert_eq!(host.data, None);
        assert_eq!(host.connection_options, None);
        assert_eq!(host.defaults.as_ref(), None);
            // serde_json::json!({
            //     "platform": "linux"
            // })
        // );
    }
    #[test]
    fn test_hosts_new() {
        let mut hosts = Hosts::new();

        // Add 10 hosts to the hosts map with dummy data
        for i in 1..=10 {
            let host = Host::builder(
                &format!("host{}.example.com", i),
                &format!("host{}.example.com", i),
            )
            .port(2200 + i as u16)
            .username(&format!("user{}", i))
            .password(&format!("password{}", i))
            .platform(if i % 2 == 0 { "linux" } else { "windows" })
            .data(vec![format!("data for host {}", i)])
            .connection_options(ConnectionOptions::new(&format!("host{}.example.com", i)))
            .build();

            // Tries to get the hosts object from the hosts map or creates an entry with an empty hashmap
            hosts.add_host(host);
        }
        assert_eq!(hosts.len(), 10);
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
}
