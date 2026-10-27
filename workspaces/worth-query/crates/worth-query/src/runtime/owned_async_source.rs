use std::collections::BTreeMap;

use worth_runtime_bridge::facade::{
    BridgeOwnedAsyncCompletionAdmission, BridgeOwnedAsyncRequestAdmission,
};

use super::{WorthQueryRuntime, WorthQueryRuntimeError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOwnedAsyncRequestDeclaration {
    identity: crate::application::WorthQueryAsyncResourceRequestIdentity,
    clause: crate::application::WorthQueryAsyncDeclarationClause,
    payload_contract: u64,
    max_payload_bytes: u64,
    retry_max_attempts: u32,
    retry_delay_ticks: u64,
    timeout_ticks: u64,
}

impl WorthQueryOwnedAsyncRequestDeclaration {
    pub fn from_async_resource_identity(
        identity: crate::application::WorthQueryAsyncResourceRequestIdentity,
        payload_contract: u64,
        max_payload_bytes: u64,
        retry_max_attempts: u32,
        retry_delay_ticks: u64,
        timeout_ticks: u64,
    ) -> Self {
        let clause = crate::application::WorthQueryAsyncDeclarationClause::resource_request(
            identity.source_family(),
            identity.loading_posture(),
            identity.failure_posture(),
            identity.request_identity().to_vec(),
        );
        Self {
            identity,
            clause,
            payload_contract,
            max_payload_bytes,
            retry_max_attempts,
            retry_delay_ticks,
            timeout_ticks,
        }
    }

    pub fn identity(&self) -> &crate::application::WorthQueryAsyncResourceRequestIdentity {
        &self.identity
    }

    pub(crate) const fn payload_contract(&self) -> u64 {
        self.payload_contract
    }
    pub(crate) const fn max_payload_bytes(&self) -> u64 {
        self.max_payload_bytes
    }
    pub(crate) const fn retry_max_attempts(&self) -> u32 {
        self.retry_max_attempts
    }
    pub(crate) const fn retry_delay_ticks(&self) -> u64 {
        self.retry_delay_ticks
    }
    pub(crate) const fn timeout_ticks(&self) -> u64 {
        self.timeout_ticks
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledOwnedAsyncDeclaration {
    source: worth_query_execution::facade::resource_custody::WorthQueryInstalledOwnedAsyncSource,
    identity: crate::application::WorthQueryAsyncResourceRequestIdentity,
    clause: crate::application::WorthQueryAsyncDeclarationClause,
}

impl WorthQueryInstalledOwnedAsyncDeclaration {
    pub fn identity(&self) -> &crate::application::WorthQueryAsyncResourceRequestIdentity {
        &self.identity
    }
    pub fn clause(&self) -> &crate::application::WorthQueryAsyncDeclarationClause {
        &self.clause
    }
    pub fn runtime_provenance(&self) -> super::WorthQueryRuntimeProvenance {
        super::WorthQueryRuntimeProvenance::from_authority(self.source.runtime_authority())
    }
}

use worth_query_execution::facade::resource_custody::WorthQueryOwnedAsyncRuntimeServices;
pub use worth_query_execution::facade::resource_custody::{
    WorthQueryOwnedAsyncRequestCustody, WorthQueryOwnedAsyncRuntimeCause,
    WorthQueryOwnedAsyncRuntimeDenial, WorthQueryOwnedAsyncRuntimeStop,
    WorthQueryOwnedAsyncRuntimeTopology,
};

pub(super) fn install_owned_async_registry(
    authority: worth_query_execution::facade::runtime::WorthQueryRuntimeAuthorityIdentity,
    product: &super::installed_product::WorthQueryInstalledProduct,
    declarations: Vec<(
        WorthQueryOwnedAsyncRequestDeclaration,
        worth_runtime_bridge::facade::LoweredBridgeAsyncSourceDeclaration,
    )>,
) -> Result<BTreeMap<String, WorthQueryInstalledOwnedAsyncDeclaration>, WorthQueryRuntimeError> {
    let services =
        WorthQueryOwnedAsyncRuntimeServices::new(authority, &product.world, &product.conditional);
    declarations
        .into_iter()
        .map(|(declaration, lowered)| {
            let source = services.installed_source(lowered).map_err(|cause| {
                WorthQueryRuntimeError::InvariantRegistration {
                    stage: "owned_async_source_installation",
                    message: format!("{cause:?}"),
                }
            })?;
            Ok((
                declaration.identity.canonical_identity().to_owned(),
                WorthQueryInstalledOwnedAsyncDeclaration {
                    source,
                    identity: declaration.identity,
                    clause: declaration.clause,
                },
            ))
        })
        .collect()
}

impl WorthQueryRuntime {
    pub(super) fn owned_async_services(&self) -> WorthQueryOwnedAsyncRuntimeServices<'_> {
        WorthQueryOwnedAsyncRuntimeServices::new(
            self.authority_identity,
            &self.installed_product.world,
            &self.installed_product.conditional,
        )
    }

    pub fn installed_owned_bridge_async_declaration(
        &self,
        identity: &crate::application::WorthQueryAsyncResourceRequestIdentity,
    ) -> Option<WorthQueryInstalledOwnedAsyncDeclaration> {
        self.installed_owned_async_declarations
            .get(identity.canonical_identity())
            .cloned()
    }

    pub fn admit_owned_bridge_async_request(
        &self,
        declaration: &WorthQueryInstalledOwnedAsyncDeclaration,
        selected: &worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
    ) -> Result<WorthQueryOwnedAsyncRequestCustody, WorthQueryOwnedAsyncRuntimeStop> {
        self.owned_async_services()
            .admit(&declaration.source, selected)
    }

    /// Uses the actual observed branch-head snapshot without claiming a commit.
    pub fn admit_owned_bridge_async_request_at_branch_head(
        &self,
        declaration: &WorthQueryInstalledOwnedAsyncDeclaration,
        selected: &worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
    ) -> Result<WorthQueryOwnedAsyncRequestCustody, WorthQueryOwnedAsyncRuntimeStop> {
        self.owned_async_services()
            .admit_at_branch_head(&declaration.source, selected)
    }

    pub fn cleanup_owned_bridge_async_entry(
        &self,
        owner: &mut WorthQueryOwnedAsyncRequestCustody,
    ) -> Result<(), WorthQueryOwnedAsyncRuntimeCause> {
        self.owned_async_services().cleanup(owner)
    }

    pub fn cancel_custodied_owned_bridge_async_request(
        &self,
        owner: &WorthQueryOwnedAsyncRequestCustody,
    ) -> Result<(), WorthQueryOwnedAsyncRuntimeCause> {
        self.owned_async_services().cancel(owner)
    }

    pub fn admit_owned_bridge_async_completion(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
        raw: worth_signal::facade::RawCompletionEnvelope,
    ) -> Result<BridgeOwnedAsyncCompletionAdmission, WorthQueryOwnedAsyncRuntimeDenial> {
        self.owned_async_services().admit_completion(request, raw)
    }

    pub fn admit_owned_bridge_async_effects_indeterminate(
        &self,
        observation: worth_runtime_bridge::facade::BridgeAsyncEffectsIndeterminateObservation,
    ) -> Result<BridgeOwnedAsyncCompletionAdmission, WorthQueryOwnedAsyncRuntimeDenial> {
        self.owned_async_services()
            .admit_effects_indeterminate(observation)
    }

    pub fn retire_owned_bridge_async_request(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
    ) -> Result<(), WorthQueryOwnedAsyncRuntimeDenial> {
        self.owned_async_services().retire_witness(request)
    }

    pub fn order_owned_bridge_async_completion(
        &self,
        completion: &BridgeOwnedAsyncCompletionAdmission,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeMixedCauseOrdering,
        WorthQueryOwnedAsyncRuntimeDenial,
    > {
        self.owned_async_services().order_completion(completion)
    }

    pub fn owned_async_runtime_topology(&self) -> Option<WorthQueryOwnedAsyncRuntimeTopology> {
        self.owned_async_services().topology(
            self.conditional_execution_registry.len(),
            self.installed_owned_async_declarations.len(),
        )
    }
}
