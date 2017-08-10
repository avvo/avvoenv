use std;
use std::result::Result;
use std::collections::HashMap;

use avvoenv::errors;
use avvoenv::source;
use avvoenv::source::consul;
use avvoenv::source::vault;

extern crate serde_json;
extern crate glob;

pub struct Env {
    map: std::collections::HashMap<String, String>,
}

impl Env {
    pub fn fetch(service: String, consul: consul::Client, vault: vault::Client, include: Vec<glob::Pattern>, exclude: Vec<glob::Pattern>, extra: HashMap<String, String>) -> Result<Env, String> {
        let mut map = std::collections::HashMap::new();

        Env::do_fetch(&consul, "global", &mut map)?;
        Env::do_fetch(&vault, "global", &mut map)?;
        match Env::get_dependencies(&consul, &service) {
            Ok(m) => map.extend(m),
            Err(errors::Error::Empty) => (),
            Err(e) => return Err(format!("error fetching from {}/{}: {}", source::Source::address(&consul), &service, e)),
        }
        match Env::get_generated(&consul, &service) {
            Ok(m) => map.extend(m),
            Err(errors::Error::Empty) => (),
            Err(e) => return Err(format!("error fetching from {}/{}: {}", source::Source::address(&consul), &service, e)),
        }
        Env::do_fetch(&consul, &service, &mut map)?;
        Env::do_fetch(&vault, &service, &mut map)?;

        map.retain(|key, _| {
            (include.is_empty() || include.iter().any(|p| p.matches(key))) &&
                !exclude.iter().any(|p| p.matches(key))
        });

        map.extend(extra);
        Ok(Env { map: map })
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
