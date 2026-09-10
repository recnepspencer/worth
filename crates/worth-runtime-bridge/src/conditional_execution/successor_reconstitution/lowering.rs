use super::{BridgeConditionalDenial, BridgeInstalledConditionalLowering};
use std::sync::Arc;
use worth_signal::facade::branch::SignalConditionalExecutionPort;

pub(super) fn readmit(
    installed: &BridgeInstalledConditionalLowering,
    service: &Arc<SignalConditionalExecutionPort<(), (), ()>>,
) -> Result<Arc<BridgeInstalledConditionalLowering>, BridgeConditionalDenial> {
    let correspondences = Arc::clone(&installed.correspondences);
    let contract = service
        .readmit_installed_contract(installed.signal_contract())
        .map_err(|denial| {
            super::reconstitution_denial(format!(
                "installed contract readmission was denied: {denial:?}"
            ))
        })?;
    let (authority, projection) =
        crate::conditional_execution::lowering_authority::mint_bridge_conditional_lowering_identity(
            crate::conditional_execution::lowering_identity::installed_lowering_identity(
                installed.bridge_runtime_key,
                installed.registry_key.branch(),
                &contract,
                installed.contract.identity(),
                &correspondences,
            ),
        );
    let lowering = Arc::new(BridgeInstalledConditionalLowering {
        bridge_runtime_key: installed.bridge_runtime_key,
        registry_key: installed.registry_key.clone(),
        _authority: authority,
        projection,
        contract: installed.contract.clone(),
        location: installed.location.clone(),
        correspondences,
        semantic_observation_plan: installed.semantic_observation_plan.clone(),
        signal_contract: std::sync::OnceLock::from(contract),
        signal_port: Default::default(),
        definition_custody: installed.definition_custody.clone(),
        providers: installed.providers.clone(),
        provider_admission: installed.provider_admission.clone(),
        lease: Arc::new(
            crate::conditional_execution::liveness::BridgeConditionalLoweringLease::issue(),
        ),
        _definition_retention: installed._definition_retention.clone(),
        counters: installed.counters,
    });
    lowering.bind_signal_port(Arc::clone(service));
    Ok(lowering)
}
