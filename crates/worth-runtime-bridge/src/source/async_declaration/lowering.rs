mod construction;
mod storage;
#[cfg(test)]
mod storage_tests;

use std::sync::Arc;
pub(crate) use storage::OriginalLoweringStorage;
use storage::{LoweredBridgeAsyncSourceDeclarationBody, LoweringBodyStorage};

use sha2::{Digest, Sha256};

use crate::identity::{AsyncSourceLoweringIdentityTag, BridgeIdentity};
use worth_signal::facade::{
    AsyncNodeCapabilityDeclaration, LoweredAsyncNodeCapabilityBundle, LoweredResourceDescriptor,
    ResourceNodeDeclaration, ResourceNodeId, SignalGraph, SignalRuntime,
};

use super::declaration::{
    BridgeAsyncSourceDeclarationBody, BridgeAsyncSourceDeclarationIdentity,
    ValidatedBridgeAsyncSourceDeclaration,
};
use super::family::{BridgeAsyncSignalLoweringFamilyKind, BridgeAsyncSourceDeclarationFamilyKind};
use super::rejection::{
    BridgeAsyncSourceDeclarationRejection, BridgeAsyncSourceDeclarationRejectionKind,
};
use super::BridgeAsyncSourceDeclarationCounters;

pub type BridgeAsyncSourceLoweringIdentity = BridgeIdentity<AsyncSourceLoweringIdentityTag>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredBridgeAsyncSourceDeclaration {
    declaration_identity: BridgeAsyncSourceDeclarationIdentity,
    family_kind: BridgeAsyncSourceDeclarationFamilyKind,
    lowering_family_kind: BridgeAsyncSignalLoweringFamilyKind,
    lowering_identity: BridgeAsyncSourceLoweringIdentity,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
    counters: BridgeAsyncSourceDeclarationCounters,
    body: Arc<LoweringBodyStorage>,
}

impl LoweredBridgeAsyncSourceDeclaration {
    pub fn lower(
        validated: &ValidatedBridgeAsyncSourceDeclaration,
    ) -> Result<Self, BridgeAsyncSourceDeclarationRejection> {
        Self::lower_with_storage(validated, None)
    }

    pub(crate) fn lower_original(
        validated: &ValidatedBridgeAsyncSourceDeclaration,
        storage: OriginalLoweringStorage,
    ) -> Result<Self, BridgeAsyncSourceDeclarationRejection> {
        Self::lower_with_storage(validated, Some(storage))
    }

    fn lower_with_storage(
        validated: &ValidatedBridgeAsyncSourceDeclaration,
        storage: Option<OriginalLoweringStorage>,
    ) -> Result<Self, BridgeAsyncSourceDeclarationRejection> {
        match validated.body() {
            BridgeAsyncSourceDeclarationBody::RequestResponse { declaration } => {
                Self::lower_request_response(validated, declaration, storage)
            }
            BridgeAsyncSourceDeclarationBody::SubscriptionBacked { declaration } => {
                Self::lower_subscription_backed(validated, declaration, storage)
            }
        }
    }

    pub fn declaration_identity(&self) -> &BridgeAsyncSourceDeclarationIdentity {
        &self.declaration_identity
    }

    pub fn declaration_identity_for_reporting(&self) -> &str {
        self.declaration_identity.as_str()
    }

    pub fn family_kind(&self) -> BridgeAsyncSourceDeclarationFamilyKind {
        self.family_kind
    }

    pub fn lowering_family_kind(&self) -> BridgeAsyncSignalLoweringFamilyKind {
        self.lowering_family_kind
    }

    pub fn lowering_identity(&self) -> &BridgeAsyncSourceLoweringIdentity {
        &self.lowering_identity
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }

    pub fn counters(&self) -> &BridgeAsyncSourceDeclarationCounters {
        &self.counters
    }

    pub fn resource_descriptor(&self) -> Option<&LoweredResourceDescriptor> {
        match &self.body.body {
            LoweredBridgeAsyncSourceDeclarationBody::RequestResponse { descriptor, .. } => {
                Some(descriptor)
            }
            LoweredBridgeAsyncSourceDeclarationBody::SubscriptionBacked { .. } => None,
        }
    }

    pub fn async_node_capability_bundle(&self) -> Option<&LoweredAsyncNodeCapabilityBundle> {
        match &self.body.body {
            LoweredBridgeAsyncSourceDeclarationBody::RequestResponse { .. } => None,
            LoweredBridgeAsyncSourceDeclarationBody::SubscriptionBacked { bundle, .. } => {
                Some(bundle)
            }
        }
    }

    pub(crate) fn instantiate_request_response(
        &self,
        instance_identity: &str,
    ) -> Result<Self, BridgeAsyncSourceDeclarationRejection> {
        let LoweredBridgeAsyncSourceDeclarationBody::RequestResponse { descriptor, .. } =
            &self.body.body
        else {
            return Err(BridgeAsyncSourceDeclarationRejection::new(
                BridgeAsyncSourceDeclarationRejectionKind::SignalDeclarationRejected,
                "bridge managed execution can instantiate only a request-response declaration",
            ));
        };
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-async-lowering-instance|template={}|instance={instance_identity}|payload-contract={}|registry-digest={}|bundle-digest={}",
            self.body.template_digest,
            descriptor.payload_contract_digest().as_str(),
            descriptor.lowered_policy_bundle().registry_digest().as_str(),
            descriptor.lowered_policy_bundle().bundle_digest().as_str(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Ok(Self {
            declaration_identity: BridgeAsyncSourceDeclarationIdentity::admit_bridge_owned(
                format!("bridge-managed-execution-declaration:{instance_identity}"),
            ),
            family_kind: self.family_kind,
            lowering_family_kind: self.lowering_family_kind,
            lowering_identity: BridgeAsyncSourceLoweringIdentity::admit_bridge_owned(format!(
                "bridge-async-lowering-instance:sha256:{digest:x}"
            )),
            canonical_basis,
            digest: Arc::from(format!("bridge-async-lowering-instance:sha256:{digest:x}")),
            counters: BridgeAsyncSourceDeclarationCounters::request_response_lowered(),
            body: Arc::clone(&self.body),
        })
    }

    fn lower_request_response(
        validated: &ValidatedBridgeAsyncSourceDeclaration,
        declaration: &ResourceNodeDeclaration,
        storage: Option<OriginalLoweringStorage>,
    ) -> Result<Self, BridgeAsyncSourceDeclarationRejection> {
        let mut runtime = signal_runtime();
        let remapped = remap_resource_declaration_to_live_graph(&mut runtime, declaration);
        runtime
            .declare_resource_node(remapped.clone())
            .map_err(signal_declaration_rejected)?;
        let descriptor = runtime
            .resource_descriptor_for_node(remapped.node())
            .ok_or_else(|| {
                BridgeAsyncSourceDeclarationRejection::new(
                    BridgeAsyncSourceDeclarationRejectionKind::SignalDeclarationRejected,
                    format!(
                        "signal runtime declared bridge async request-response source `{}` but did not retain a lowered resource descriptor for node {}",
                        validated.declaration_identity().as_str(),
                        declaration.node().node()
                    ),
                )
            })?;

        Ok(Self::request_response_parts(
            validated,
            remapped,
            descriptor.clone(),
            storage,
        ))
    }

    fn lower_subscription_backed(
        validated: &ValidatedBridgeAsyncSourceDeclaration,
        declaration: &AsyncNodeCapabilityDeclaration,
        storage: Option<OriginalLoweringStorage>,
    ) -> Result<Self, BridgeAsyncSourceDeclarationRejection> {
        let mut runtime = signal_runtime();
        let remapped = remap_async_node_declaration_to_live_graph(&mut runtime, declaration);
        runtime
            .declare_async_node_capability(remapped.clone())
            .map_err(signal_declaration_rejected)?;
        let bundle = runtime
            .async_node_capability_bundle_for_node(remapped.node())
            .ok_or_else(|| {
                BridgeAsyncSourceDeclarationRejection::new(
                    BridgeAsyncSourceDeclarationRejectionKind::SignalDeclarationRejected,
                    format!(
                        "signal runtime declared bridge async subscription-backed source `{}` but did not retain an async capability bundle for node {}",
                        validated.declaration_identity().as_str(),
                        declaration.node()
                    ),
                )
            })?;

        Ok(Self::new_subscription_backed(
            validated, remapped, bundle, storage,
        ))
    }

    pub(crate) fn request_response_declaration(&self) -> Option<&ResourceNodeDeclaration> {
        match &self.body.body {
            LoweredBridgeAsyncSourceDeclarationBody::RequestResponse { declaration, .. } => {
                Some(declaration)
            }
            LoweredBridgeAsyncSourceDeclarationBody::SubscriptionBacked { .. } => None,
        }
    }

    pub(crate) fn subscription_backed_declaration(
        &self,
    ) -> Option<&AsyncNodeCapabilityDeclaration> {
        match &self.body.body {
            LoweredBridgeAsyncSourceDeclarationBody::RequestResponse { .. } => None,
            LoweredBridgeAsyncSourceDeclarationBody::SubscriptionBacked { declaration, .. } => {
                Some(declaration)
            }
        }
    }
}

fn signal_runtime() -> SignalRuntime<(), (), (), (), ()> {
    SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build()
}

pub(crate) fn remap_resource_declaration_to_live_graph(
    runtime: &mut SignalRuntime<(), (), (), (), ()>,
    declaration: &ResourceNodeDeclaration,
) -> ResourceNodeDeclaration {
    let live_node = ResourceNodeId::from_node(runtime.graph_mut().node().build());
    let live_dependents = declaration
        .declared_dependent_cancellation_nodes()
        .iter()
        .map(|_| ResourceNodeId::from_node(runtime.graph_mut().node().build()));

    let mut remapped =
        ResourceNodeDeclaration::new(live_node, declaration.payload_contract().clone())
            .with_lifecycle_policy(declaration.lifecycle_policy())
            .with_retry_policy(declaration.retry_policy().clone())
            .with_timeout_policy(declaration.timeout_policy().clone())
            .with_cancellation_policy(declaration.cancellation_policy().clone())
            .with_stale_after_policy(declaration.stale_after_policy().clone())
            .with_supersession_policy(declaration.supersession_policy().clone())
            .with_revalidation_policy(declaration.revalidation_policy().clone())
            .with_observation_policy(declaration.observation_policy().clone())
            .with_output_continuity_policy(declaration.output_continuity_policy().clone())
            .with_retention_policy(declaration.retention_policy().clone())
            .with_diagnostics_policy(declaration.diagnostics_policy().clone())
            .with_replay_policy(declaration.replay_policy().clone())
            .with_declared_dependent_cancellation_nodes(live_dependents);

    if let Some(grace_period) = declaration.cancellation_grace_period() {
        remapped = remapped.with_cancellation_grace_period(grace_period);
    }
    if let Some(max_attempts) = declaration.retry_max_attempts() {
        remapped = remapped.with_retry_max_attempts(max_attempts);
    }
    if let Some(max_jitter) = declaration.retry_deterministic_jitter() {
        remapped = remapped.with_retry_deterministic_jitter(max_jitter);
    }
    if let (Some(scope), Some(limit)) = (
        declaration.retry_budget_scope(),
        declaration.retry_budget_limit(),
    ) {
        remapped = remapped.with_retry_budget(scope, limit);
    }

    remapped
}

pub(crate) fn remap_async_node_declaration_to_live_graph(
    runtime: &mut SignalRuntime<(), (), (), (), ()>,
    declaration: &AsyncNodeCapabilityDeclaration,
) -> AsyncNodeCapabilityDeclaration {
    let remapped = remap_resource_declaration_to_live_graph(
        runtime,
        &declaration.clone().into_legacy_resource_declaration(),
    );
    AsyncNodeCapabilityDeclaration::from_legacy_resource_declaration(remapped)
}

fn signal_declaration_rejected(
    error: worth_signal::facade::SignalError,
) -> BridgeAsyncSourceDeclarationRejection {
    BridgeAsyncSourceDeclarationRejection::new(
        BridgeAsyncSourceDeclarationRejectionKind::SignalDeclarationRejected,
        error.to_string(),
    )
}
