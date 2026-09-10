use std::collections::BTreeMap;

use worth_runtime_bridge::facade::{
    BridgeAsyncCompletionRejection, BridgeAsyncRequestIdentityRejection,
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
    runtime_provenance: super::WorthQueryRuntimeProvenance,
    signal_graph_instance: u64,
    identity: crate::application::WorthQueryAsyncResourceRequestIdentity,
    clause: crate::application::WorthQueryAsyncDeclarationClause,
    lowered: worth_runtime_bridge::facade::LoweredBridgeAsyncSourceDeclaration,
}

impl WorthQueryInstalledOwnedAsyncDeclaration {
    pub fn identity(&self) -> &crate::application::WorthQueryAsyncResourceRequestIdentity {
        &self.identity
    }
    pub fn clause(&self) -> &crate::application::WorthQueryAsyncDeclarationClause {
        &self.clause
    }
    pub fn runtime_provenance(&self) -> super::WorthQueryRuntimeProvenance {
        self.runtime_provenance
    }
    pub(super) const fn signal_graph_instance(&self) -> u64 {
        self.signal_graph_instance
    }
    pub(super) fn lowered_declaration_identity(
        &self,
    ) -> &worth_runtime_bridge::facade::BridgeAsyncSourceDeclarationIdentity {
        self.lowered.declaration_identity()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryOwnedAsyncRuntimeDenial {
    ForeignRuntime,
    SuccessorRuntime,
    ProductSelectionMismatch,
    Request(BridgeAsyncRequestIdentityRejection),
    Completion(BridgeAsyncCompletionRejection),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryOwnedAsyncRuntimeTopology {
    signal_graph_instance: u64,
    installed_conditional_nodes: usize,
    installed_async_declarations: usize,
    active_signal_nodes: usize,
}

impl WorthQueryOwnedAsyncRuntimeTopology {
    pub const fn signal_graph_instance(self) -> u64 {
        self.signal_graph_instance
    }
    pub const fn installed_conditional_nodes(self) -> usize {
        self.installed_conditional_nodes
    }
    pub const fn installed_async_declarations(self) -> usize {
        self.installed_async_declarations
    }
    pub const fn active_signal_nodes(self) -> usize {
        self.active_signal_nodes
    }
}

pub(super) fn install_owned_async_registry(
    authority: worth_query_execution::facade::runtime::WorthQueryRuntimeAuthorityIdentity,
    product: &super::installed_product::WorthQueryInstalledProduct,
    declarations: Vec<(
        WorthQueryOwnedAsyncRequestDeclaration,
        worth_runtime_bridge::facade::LoweredBridgeAsyncSourceDeclaration,
    )>,
) -> Result<BTreeMap<String, WorthQueryInstalledOwnedAsyncDeclaration>, WorthQueryRuntimeError> {
    if declarations.is_empty() {
        return Ok(BTreeMap::new());
    }
    let signal_graph_instance = product.conditional.owned_signal_graph_instance_id();
    Ok(declarations
        .into_iter()
        .map(|(declaration, lowered)| {
            let key = declaration.identity.canonical_identity().to_owned();
            let installed = WorthQueryInstalledOwnedAsyncDeclaration {
                runtime_provenance: super::WorthQueryRuntimeProvenance::from_authority(authority),
                signal_graph_instance,
                identity: declaration.identity,
                clause: declaration.clause,
                lowered,
            };
            (key, installed)
        })
        .collect())
}

impl WorthQueryRuntime {
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
    ) -> Result<BridgeOwnedAsyncRequestAdmission, WorthQueryOwnedAsyncRuntimeDenial> {
        if declaration.runtime_provenance != self.runtime_provenance() {
            return Err(WorthQueryOwnedAsyncRuntimeDenial::ForeignRuntime);
        }
        let product = &self.installed_product;
        if declaration.signal_graph_instance != product.conditional.owned_signal_graph_instance_id()
        {
            return Err(WorthQueryOwnedAsyncRuntimeDenial::SuccessorRuntime);
        }
        product
            .validate_selected_source(selected, None)
            .map_err(|_| WorthQueryOwnedAsyncRuntimeDenial::ProductSelectionMismatch)?;
        product
            .world
            .admit_owned_async_request(&product.conditional, &declaration.lowered, selected)
            .map_err(|denial| match denial {
                worth_query_execution::facade::integration::RuntimeWorldOwnedAsyncRequestAdmissionDenial::ForeignOwner
                | worth_query_execution::facade::integration::RuntimeWorldOwnedAsyncRequestAdmissionDenial::RelationalSourceMismatch => {
                    WorthQueryOwnedAsyncRuntimeDenial::ProductSelectionMismatch
                }
                worth_query_execution::facade::integration::RuntimeWorldOwnedAsyncRequestAdmissionDenial::Bridge(
                    denial,
                ) => WorthQueryOwnedAsyncRuntimeDenial::Request(denial),
            })
    }

    pub fn admit_owned_bridge_async_completion(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
        raw: worth_signal::facade::RawCompletionEnvelope,
    ) -> Result<BridgeOwnedAsyncCompletionAdmission, WorthQueryOwnedAsyncRuntimeDenial> {
        let runtime = &self.installed_product.conditional;
        let validated = runtime
            .validate_owned_async_completion_envelope(request, raw)
            .map_err(WorthQueryOwnedAsyncRuntimeDenial::Completion)?;
        runtime
            .admit_owned_async_completion(request, &validated)
            .map_err(WorthQueryOwnedAsyncRuntimeDenial::Completion)
    }

    pub fn admit_owned_bridge_async_effects_indeterminate(
        &self,
        observation: worth_runtime_bridge::facade::BridgeAsyncEffectsIndeterminateObservation,
    ) -> Result<BridgeOwnedAsyncCompletionAdmission, WorthQueryOwnedAsyncRuntimeDenial> {
        self.installed_product
            .conditional
            .admit_owned_async_effects_indeterminate(observation)
            .map_err(WorthQueryOwnedAsyncRuntimeDenial::Completion)
    }

    pub fn retire_owned_bridge_async_request(
        &self,
        request: &BridgeOwnedAsyncRequestAdmission,
    ) -> Result<(), WorthQueryOwnedAsyncRuntimeDenial> {
        self.installed_product
            .conditional
            .retire_owned_async_request(request)
            .map(|_| ())
            .map_err(WorthQueryOwnedAsyncRuntimeDenial::Completion)
    }

    pub fn order_owned_bridge_async_completion(
        &self,
        completion: &BridgeOwnedAsyncCompletionAdmission,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeMixedCauseOrdering,
        WorthQueryOwnedAsyncRuntimeDenial,
    > {
        self.installed_product
            .conditional
            .order_owned_async_completion_report(completion)
            .map_err(WorthQueryOwnedAsyncRuntimeDenial::Completion)
    }

    pub fn owned_async_runtime_topology(&self) -> Option<WorthQueryOwnedAsyncRuntimeTopology> {
        let product = &self.installed_product;
        product
            .conditional
            .owned_signal_active_node_count()
            .ok()
            .map(|active_signal_nodes| WorthQueryOwnedAsyncRuntimeTopology {
                signal_graph_instance: product.conditional.owned_signal_graph_instance_id(),
                installed_conditional_nodes: self.conditional_execution_registry.len(),
                installed_async_declarations: self.installed_owned_async_declarations.len(),
                active_signal_nodes,
            })
    }
}
