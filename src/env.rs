use std::{
    collections::HashMap,
    env, fmt,
    io::{self, Read},
    path::PathBuf,
};

use dirs::home_dir;
use log::{debug, info, trace, warn};
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
    VaultTokenError(vault::ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ConsulError(e) => e.fmt(f),
            Error::IoError(e) => e.fmt(f),
            Error::RancherError(e) => e.fmt(f),
            Error::ServiceError(e) => e.fmt(f),
            Error::VaultError(e) => e.fmt(f),
            Error::VaultTokenError(e) => e.fmt(f),
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
            Error::VaultTokenError(e) => Some(e),
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

impl From<vault::ParseError> for Error {
    fn from(e: vault::ParseError) -> Error {
        Error::VaultTokenError(e)
    }
}

#[derive(Deserialize)]
struct VersionInfo {
    version: u64,
}

pub(crate) fn fetch(opts: FetchOpts) -> Result<HashMap<String, String>, Error> {
    let mut env = HashMap::new();
    let service = service::name(opts.service)?;
    info!("Fetching environment for {}", service);

    let consul = consul::Client::new(opts.consul)?;
    trace!("Configured Consul: {:?}", consul);
    let mut vault = vault::Client::new(opts.vault)?;
    trace!("Configured Vault: {:?}", vault);

    if opts.dev {
        info!("Authenticating with Vault via LDAP");
        let user = prompt_default("Vault username: ", env::var("USER").ok())?;
        let password = prompt_password("Vault password: ")?;
        vault.ldap_auth(&user, &password)?;
    } else if let (Some(app_id), Some(app_user)) = (&opts.app_id, &opts.app_user) {
        debug!("Authenticating with Vault via App ID");
        vault.app_id_auth(app_id, app_user)?;
    } else if let Some(token) = opts.token {
        debug!("Using supplied Vault token");
        vault.token(token);
    } else {
        debug!("Using Vault token from ~/.vault-token");
        let mut path = home_dir().unwrap_or_else(|| PathBuf::from("/"));
        path.push(".vault-token");
        let f = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(f);
        let mut string = String::new();
        reader.read_to_string(&mut string)?;
        vault.token(string.trim().parse()?);
    }

    if !opts.skip_rancher_metadata && rancher_metadata::is_available() {
        debug!("Fetching config from Rancher");
        let rancher = rancher_metadata::Client::new();
        if let Some(info) = rancher.info()? {
            let map: HashMap<_, _> = info.into_iter().collect();
            trace!("Merging to environment: {:?}", map);
            env.extend(map);
        }
    }

    debug!("Fetching global config");
    fill(&mut env, &consul, "global")?;
    debug!("Fetching global secrets");
    fill(&mut env, &vault, "global")?;

    debug!("Fetching {} dependencies", service);
    fill_dependencies(&mut env, &consul, &service)?;
    debug!("Fetching {} generated", service);
    fill_generated(&mut env, &consul, &service)?;

    debug!("Fetching {} config", service);
    fill(&mut env, &consul, &service)?;
    debug!("Fetching {} secrets", service);
    fill(&mut env, &vault, &service)?;

    let include = opts.include;
    let exclude = opts.exclude;
    env.retain(|key, _| {
        let keep = (include.is_empty() || include.iter().any(|p| p.matches(key)))
            && !exclude.iter().any(|p| p.matches(key));
        if !keep {
            trace!("Filtering out {:?}", key);
        }
        keep
    });

    trace!("Merging to environment from options: {:?}", opts.add);
    env.extend(opts.add);

    Ok(env)
}

fn fill<T>(env: &mut HashMap<String, String>, client: &T, service: &str) -> Result<(), Error>
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
    debug!("Got version {}", version);
    if let Some(mut map) =
        client.get::<HashMap<String, String>>(&format!("config/{}/{}", service, version))?
    {
        map.remove("__timestamp__");
        map.remove("__user__");
        trace!("Merging to environment: {:?}", map);
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
    trace!("Got dependencies: {:?}", deps);
    for dep in deps {
        let key = format!("{}_BASE_URL", dep.replace("-", "_").to_uppercase());
        match client.get::<String>(&format!("infrastructure/service-uris/{}", key)) {
            Ok(Some(val)) => {
                trace!("Merging to environment: {:?}: {:?}", key, val);
                env.insert(key, val);
            }
            Ok(None) => warn!("Missing URL for {}", dep),
            Err(e) => return Err(e.into()),
        };

        let frontend_key = format!("{}_FRONTEND_URL", dep.replace("-", "_").to_uppercase());
        match client.get::<String>(&format!("infrastructure/service-uris/{}", frontend_key)) {
            Ok(Some(val)) => {
                trace!("Merging to environment: {:?}: {:?}", frontend_key, val);
                env.insert(frontend_key, val);
            }
            Ok(None) => info!("Frontend URL for {} either not needed or not set", dep),
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
        trace!("Merging to environment: {:?}", generated);
        env.extend(generated);
    }
    Ok(())
}
