use std;
use std::result::Result;
use std::collections::HashMap;

use avvoenv::errors;
use avvoenv::source;
use avvoenv::source::consul;

extern crate hyper;

pub struct Env {
    consul: consul::Client,
    map: std::collections::HashMap<String, String>
}

impl Env {
    fn new(consul: &str) -> Result<Env, errors::Error> {
        let consul_client = consul::Client::new(consul, "config/")?;
        Ok(Env { consul: consul_client, map: HashMap::new() })
    }

    pub fn fetch(consul: &str) -> Result<Env, errors::Error> {
        let mut env = Env::new(consul)?;
        env.do_fetch()?;
        Ok(env)
    }

    fn do_fetch(&mut self) -> Result<(), errors::Error> {
        match Env::get_current(&self.consul, "global") {
            Ok(mut data) => {
                data.remove("__timestamp__");
                data.remove("__user__");
                self.map.extend(data);
            }
            Err(errors::Error::Empty) => (),
            Err(e) => return Err(e),
        }
        Ok(())
    }

    // TODO make source trait so this can be used for more than just consul
    fn get_current(source: &consul::Client, app: &str) -> Result<HashMap<String, String>, errors::Error> {
        let key = match source.get(&format!("{}/current", app))? {
            source::Version { version, .. } => format!("{}/{}", app, version),
        };
        source.get(&key)
    }

    pub fn vars(&self) -> std::collections::hash_map::Iter<String, String> {
        self.map.iter()
    }
}
