/*
Copyright 2024 The Kubernetes Authors.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use ingress_pingora::Context;
use ingress_pingora::controller;
use kube::Client;
use tracing::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run().await;
    Ok(())
}

pub async fn run() {
    // Defines the logging as fmt, and sets it as global logger
    // Without it, no logging will be initialized
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // Initialize a Kubernetes client.
    // The client can use a cloned subscriber
    // TODO: how can we set the log level?
    info!("Initializing Kubernetes Client");
    let client = Client::try_default()
        .await
        .expect("failed to create kube Client");

    // Copied from ChatGPT, trying to add a shared map with all ingresses we care about endpointslices
    // and eventually secrets/sects
    let eps_hashmap: HashMap<String, String> = HashMap::new();
    let eps_map = Arc::new(RwLock::new(eps_hashmap));

    let secret_hashmap: HashMap<String, String> = HashMap::new();
    let secret_map = Arc::new(RwLock::new(secret_hashmap));

    let ctx = Context {
        client: client.clone(),
        epsmap: eps_map,
        secretmap: secret_map,
    };

    if let Err(error) = controller::controller(ctx).await {
        error!("failed to start Gateway controller: {error:?}");
        std::process::exit(1);
    }
}
