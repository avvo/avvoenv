use std::{collections::HashMap, io::{self, Read}, fmt, path::PathBuf};

use dirs::home_dir;
use log::warn;
use serde::Deserialize;

use crate::{
    consul,
    prompt::{prompt_default, prompt_password},
    rancher_metadata, service, vault, FetchOpts,
};

pub trait Client {
    type Error: std::error::Error + 'static;

    fn get<T>(&self, key: &str) -> Result<Option<T>, Self::Error>
    where
        T: serde::de::DeserializeOwned + 'static;
}

#[derive(Debug)]
pub enum Error {
    ConsulError(consul::Error),
    IoError(io::Error),
    RancherError(rancher_metadata::Error),
    ServiceError(service::Error),
    VaultError(vault::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ConsulError(e) => e.fmt(f),
            Error::IoError(e) => e.fmt(f),
            Error::RancherError(e) => e.fmt(f),
            Error::ServiceError(e) => e.fmt(f),
            Error::VaultError(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::ConsulError(e) => Some(e),
            Error::IoError(e) => Some(e),
            Error::RancherError(e) => Some(e),
            Error::ServiceError(e) => Some(e),
            Error::VaultError(e) => Some(e),
        }
    }
}

impl From<consul::Error> for Error {
    fn from(e: consul::Error) -> Error {
        Error::ConsulError(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IoError(e)
    }
}

impl From<rancher_metadata::Error> for Error {
    fn from(e: rancher_metadata::Error) -> Error {
        Error::RancherError(e)
    }
}

impl From<service::Error> for Error {
    fn from(e: service::Error) -> Error {
        Error::ServiceError(e)
    }
}

impl From<vault::Error> for Error {
    fn from(e: vault::Error) -> Error {
        Error::VaultError(e)
    }
}

#[derive(Deserialize)]
struct VersionInfo {
    version: u64,
}

pub(crate) fn fetch(
    opts: FetchOpts,
) -> Result<HashMap<String, String>, Error> {
    let mut env = HashMap::new();
    let service = service::name(opts.service)?;

    let rancher = rancher_metadata::Client::new();
    let consul = consul::Client::new(opts.consul)?;
    let mut vault = vault::Client::new(opts.vault)?;

    if opts.dev {
        let user = prompt_default("Vault username: ", std::env::var("USER").ok())?;
        let password = prompt_password("Vault password: ")?;
        vault.ldap_auth(&user, &password)?;
    } else if let (Some(app_id), Some(app_user)) = (&opts.app_id, &opts.app_user) {
        vault.app_id_auth(app_id, app_user)?;
    } else if let Some(token) = opts.token {
        vault.token(token);
    } else {
        let mut path = home_dir().unwrap_or(PathBuf::from("/"));
        path.push(".vault-token");
        let f = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(f);
        let mut string = String::new();
        reader.read_to_string(&mut string)?;
        vault.token(string.trim_right().to_string());
    }

    if let Some(info) = rancher.info()? {
        env.extend(info);
    }

    fill(&mut env, &consul, "global")?;
    fill(&mut env, &vault, "global")?;

    fill_dependencies(&mut env, &consul, &service)?;
    fill_generated(&mut env, &consul, &service)?;

    fill(&mut env, &consul, &service)?;
    fill(&mut env, &vault, &service)?;

    let include = opts.include;
    let exclude = opts.exclude;
    env.retain(|key, _| {
        (include.is_empty() || include.iter().any(|p| p.matches(key)))
            && !exclude.iter().any(|p| p.matches(key))
    });

    env.extend(opts.add);

    Ok(env)
}

fn fill<T>(
    env: &mut HashMap<String, String>,
    client: &T,
    service: &str,
) -> Result<(), Error>
where
    T: Client,
    Error: From<<T as Client>::Error>,
{
    let version = client
        .get::<VersionInfo>(&format!("config/{}/current", service))?
        .map(|v| v.version)
        .unwrap_or_else(|| {
            warn!("could not determine version, using 1");
            1
        });
    if let Some(mut map) =
        client.get::<HashMap<String, String>>(&format!("config/{}/{}", service, version))?
    {
        map.remove("__timestamp__");
        map.remove("__user__");
        env.extend(map);
    };
    Ok(())
}

fn fill_dependencies(
    env: &mut HashMap<String, String>,
    client: &consul::Client,
    app: &str,
) -> Result<(), Error> {
    let deps = match client.get::<Vec<String>>(&format!("config/{}/dependencies", app))? {
        Some(v) => v,
        None => return Ok(()),
    };
    for dep in deps {
        let key = format!("{}_", dep.replace("-", "_").to_uppercase());
        match client.get::<String>(&format!("infrastructure/service-uris/{}_BASE_URL", key)) {
            Ok(Some(val)) => {
                env.insert(key, val);
            }
            Ok(None) => (),
            Err(e) => return Err(e.into()),
        };
    }
    Ok(())
}

fn fill_generated(
    env: &mut HashMap<String, String>,
    client: &consul::Client,
    app: &str,
) -> Result<(), Error> {
    if let Some(generated) =
        client.get::<HashMap<String, String>>(&format!("config/{}/generated", app))?
    {
        env.extend(generated);
    }
    Ok(())
}
