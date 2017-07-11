use std;
use std::result::Result;
use std::collections::HashMap;

use avvoenv::errors;
use avvoenv::source;
use avvoenv::source::consul;
use avvoenv::source::vault;

extern crate hyper;
extern crate serde_json;

pub struct Env {
    consul: consul::Client,
    vault: vault::Client,
    map: std::collections::HashMap<String, String>
}

impl Env {
    fn new(consul: consul::Client, vault: vault::Client, map: HashMap<String, String>) -> Env {
        Env { consul: consul, vault: vault, map: map }
    }

    pub fn fetch(service: String, consul: consul::Client, vault: vault::Client, map: HashMap<String, String>) -> Result<Env, String> {
        let mut env = Env::new(consul, vault, map);
        Env::do_fetch(&env.consul, "global", &mut env.map)?;
        Env::do_fetch(&env.vault, "global", &mut env.map)?;
        match Env::get_dependencies(&env.consul, &service) {
            Ok(map) => env.map.extend(map),
            Err(errors::Error::Empty) => (),
            Err(e) => return Err(format!("error fetching from {}/{}: {}", source::Source::address(&env.consul), &service, e)),
        }
        match Env::get_generated(&env.consul, &service) {
            Ok(map) => env.map.extend(map),
            Err(errors::Error::Empty) => (),
            Err(e) => return Err(format!("error fetching from {}/{}: {}", source::Source::address(&env.consul), &service, e)),
        }
        Env::do_fetch(&env.consul, &service, &mut env.map)?;
        Env::do_fetch(&env.vault, &service, &mut env.map)?;
        Ok(env)
    }

    pub fn vars(&self) -> &HashMap<String, String> {
        &self.map
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
            Err(e) => return Err(format!("error fetching from {}/config/{}: {}", source.address(), namespace, e)),
        }
        Ok(())
    }

    fn get_current<T>(source: &T, app: &str) -> Result<HashMap<String, String>, errors::Error>
        where T: source::Source {
        let result: serde_json::value::Value = source.get(&format!("config/{}/current", app))?;
        let version = match result["version"].as_u64() {
            Some(val) => val,
            None => return Err(errors::Error::BadVersion),
        };
        source.get(&format!("config/{}/{}", app, version))
    }

    fn get_dependencies<T>(source: &T, app: &str) -> Result<HashMap<String, String>, errors::Error>
        where T: source::Source {
        let deps: Vec<String> = source.get(&format!("config/{}/dependencies", app))?;
        let keys = deps.iter().map(|key| format!("{}_BASE_URL", key.replace("-", "_").to_uppercase()));
        let mut map = HashMap::new();
        for key in keys {
            match source.get_string(&format!("infrastructure/service-uris/{}", key)) {
                Ok(val) => {
                    map.insert(key, val);
                }
                Err(errors::Error::Empty) => (),
                Err(e) => return Err(e),
            };
        };
        Ok(map)
    }

    fn get_generated<T>(source: &T, app: &str) -> Result<HashMap<String, String>, errors::Error>
        where T: source::Source {
        source.get(&format!("config/{}/generated", app))
    }
}
