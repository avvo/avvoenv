use std;
use std::result::Result;
use std::collections::HashMap;

use avvoenv::errors;
use avvoenv::source;
use avvoenv::source::consul;
use avvoenv::source::vault;
use avvoenv::source::Source;

extern crate hyper;

pub struct Env {
    consul: consul::Client,
    vault: vault::Client,
    map: std::collections::HashMap<String, String>
}

impl Env {
    fn new(consul: hyper::Url, vault: hyper::Url, vault_token: String) -> Result<Env, String> {
        let consul_client = consul::Client::new(consul, String::from("config"))?;
        let vault_client = vault::Client::new(vault, String::from("config"), vault_token)?;
        Ok(Env { consul: consul_client, vault: vault_client, map: HashMap::new() })
    }

    pub fn fetch(service: String, consul: hyper::Url, vault: hyper::Url, vault_token: String) -> Result<Env, String> {
        let mut env = Env::new(consul, vault, vault_token)?;
        Env::do_fetch(&env.consul, "global", &mut env.map)?;
        Env::do_fetch(&env.vault, "global", &mut env.map)?;
        Env::do_fetch(&env.consul, &service, &mut env.map)?;
        Env::do_fetch(&env.vault, &service, &mut env.map)?;
        Ok(env)
    }

    fn do_fetch<T>(source: &T, namespace: &str, map: &mut std::collections::HashMap<String, String>) -> Result<(), String>
        where T: source::Source {
        match Env::get_current(source, namespace) {
            Ok(mut data) => {
                data.remove("__timestamp__");
                data.remove("__user__");
                map.extend(data);
            }
            Err(errors::Error::Empty) => (),
            Err(e) => return Err(format!("error fetching from {}: {}", source.address(), e)),
        }
        Ok(())
    }

    fn get_current<T>(source: &T, app: &str) -> Result<HashMap<String, String>, errors::Error>
    where T: source::Source {
        let key = match source.get(&format!("{}/current", app))? {
            source::Version { version, .. } => format!("{}/{}", app, version),
        };
        source.get(&key)
    }

    pub fn vars(&self) -> std::collections::hash_map::Iter<String, String> {
        self.map.iter()
    }
}
