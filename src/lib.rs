use kube::Client;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use thiserror::Error;

pub mod controller;

// Context for our reconciler
// On Rust, instead of using context.Context or creating some uber struct,
// we can define a structure that should carry the things we care. And this structure
// can be clonable
// As an example, we have defined a Kubernetes client on main, and we want to be able
// to use it on our reconciliation processes, so to pass through borrow checker
// we pass the context, and allow it to be cloned.
#[derive(Clone)]
pub struct Context {
    /// Kubernetes client
    pub client: Client,
    /// The service hash map
    /// The key of the map is a namespace/name service, tbd if the value will be a vec of ingress objects
    pub epsmap: Arc<RwLock<HashMap<String, String>>>,
    /// The certificate map
    pub secretmap: Arc<RwLock<HashMap<String, String>>>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("kube error: {0}")]
    KubeError(#[source] kube::Error),
    #[error("invalid configuration: `{0}`")]
    InvalidConfigError(String),
    #[error("Bad service to reconcile: `{0}`")]
    BadService(String),
    #[error("service listing error: {0}")]
    ServiceError(#[source] kube::Error),
    #[error("ingress listing error: {0}")]
    IngressError(#[source] kube::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
