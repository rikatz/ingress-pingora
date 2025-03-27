use futures::StreamExt;
use k8s_openapi::api::{discovery::v1::EndpointSlice, networking::v1::Ingress};
use kube::{
    Resource, ResourceExt,
    api::{Api, ListParams},
    runtime::{Controller, controller::Action, reflector::ObjectRef, watcher::Config},
};

use std::{sync::Arc, time::Duration};
use tracing::*;

use crate::*;

const SERVICE_LABEL_KEY: &str = "kubernetes.io/service-name";

pub async fn controller(ctx: Context) -> Result<()> {
    let ingress = Api::<Ingress>::all(ctx.client.clone());
    ingress
        .list(&ListParams::default().limit(1))
        .await
        .map_err(Error::IngressError)?;

    // We watch also endpoint slices to reflect the ingress endpoints
    // We don't and wont support endpoints, or external services at this point :)
    let eps = Api::<EndpointSlice>::all(ctx.client.clone());
    let epsmapper = |epsobj: EndpointSlice| {
        let svc_name = epsobj.labels().get(SERVICE_LABEL_KEY)?;
        let svc_namespace = epsobj.meta().namespace.as_ref()?;
        info!("Adding EPS {svc_namespace}/{svc_name} to the reconciliation queue");
        // Reconcile the ingress of the same service. This is mostly wrong as it assumes the service will have the
        // same name of ingress, but we will soon have a map of all services that we care, populate it during start and
        // reconcile based on it
        // In a close future we will need to reconcile a list of objects and not just one
        /*vec![
            ObjectRef::new("app-deployment-1").within(&svc_namespace),
            ObjectRef::new("app-deployment-2").within(&svc_namespace),
        ]*/
        Some(ObjectRef::new(svc_name).within(svc_namespace))
    };

    Controller::new(ingress, Config::default().any_semantic())
        .watches(eps, Config::default().any_semantic(), epsmapper)
        .shutdown_on_signal()
        .run(reconcile, error_policy, Arc::new(ctx))
        .filter_map(|x| async move { std::result::Result::ok(x) })
        .for_each(|_| futures::future::ready(()))
        .await;

    Ok(())
}

pub async fn reconcile(ingress: Arc<Ingress>, _: Arc<Context>) -> Result<Action> {
    let name = ingress
        .metadata
        .name
        .clone()
        .ok_or(Error::InvalidConfigError("invalid name".to_string()))?;
    let ns = ingress
        .metadata
        .namespace
        .clone()
        .ok_or(Error::InvalidConfigError("invalid namespace".to_string()))?;

    info!("reconciling ingress {ns}/{name}");

    Ok(Action::requeue(Duration::from_secs(60)))
}

// error_policy is the function called when a reconciliation error happens
fn error_policy(_: Arc<Ingress>, error: &Error, _: Arc<Context>) -> Action {
    println!(
        "error happened during reconciliation of ingress {:?}",
        error
    );
    Action::requeue(Duration::from_secs(5 * 60)) // Requeue after 5 minutes
}
