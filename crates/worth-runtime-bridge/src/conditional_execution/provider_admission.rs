use super::provider_semantics::BridgeConditionalProviderSemanticContracts;
use super::{
    BridgeConditionalContract, BridgeConditionalDenial, BridgeConditionalDenialKind,
    BridgeConditionalProviderSet,
};
use std::sync::Arc;

#[derive(Clone)]
pub(super) struct BridgeConditionalProviderAdmission {
    contract: Arc<BridgeConditionalContract>,
    required_roles: [bool; PROVIDER_DIMENSION_CHECK_COUNT],
    semantic_contracts: BridgeConditionalProviderSemanticContracts,
}

impl BridgeConditionalProviderAdmission {
    pub(super) fn contract(&self) -> &BridgeConditionalContract {
        &self.contract
    }
    pub(super) fn shared_contract(&self) -> &Arc<BridgeConditionalContract> {
        &self.contract
    }

    pub(super) const fn required_roles(&self) -> &[bool; PROVIDER_DIMENSION_CHECK_COUNT] {
        &self.required_roles
    }

    pub(super) fn semantic_contracts(&self) -> &BridgeConditionalProviderSemanticContracts {
        &self.semantic_contracts
    }
}

pub(super) fn admit_provider_set(
    contract: Arc<BridgeConditionalContract>,
    providers: &BridgeConditionalProviderSet,
) -> Result<BridgeConditionalProviderAdmission, BridgeConditionalDenial> {
    let required_roles = admit_provider_roles(&contract, providers)?;
    Ok(BridgeConditionalProviderAdmission {
        contract,
        required_roles,
        semantic_contracts: providers.semantic_contracts.clone(),
    })
}

pub(super) fn admit_provider_roles(
    contract: &BridgeConditionalContract,
    providers: &BridgeConditionalProviderSet,
) -> Result<[bool; PROVIDER_DIMENSION_CHECK_COUNT], BridgeConditionalDenial> {
    let required_roles = required_provider_roles(contract);
    let present_roles = present_provider_roles(providers);
    for ((required, present), (missing, extra)) in required_roles
        .into_iter()
        .zip(present_roles)
        .zip(PROVIDER_ROLE_DENIALS)
    {
        require_exact(required, present, missing, extra)?;
    }
    Ok(required_roles)
}

fn required_provider_roles(
    contract: &BridgeConditionalContract,
) -> [bool; PROVIDER_DIMENSION_CHECK_COUNT] {
    [
        contract.requires_condition_provider(),
        contract.requires_dependency_comparator(),
        contract.requires_output_comparator(),
        contract.requires_reuse_comparator(),
        contract.requires_trigger_provider(),
        contract.requires_wake_provider(),
        true,
    ]
}

fn present_provider_roles(
    providers: &BridgeConditionalProviderSet,
) -> [bool; PROVIDER_DIMENSION_CHECK_COUNT] {
    [
        providers.condition.is_some(),
        providers.dependency_comparator.is_some(),
        providers.output_comparator.is_some(),
        providers.reuse_comparator.is_some(),
        providers.trigger.is_some(),
        providers.wake.is_some(),
        providers.compute.is_some(),
    ]
}

fn require_exact(
    required: bool,
    present: bool,
    missing: BridgeConditionalDenialKind,
    extra: BridgeConditionalDenialKind,
) -> Result<(), BridgeConditionalDenial> {
    match (required, present) {
        (true, false) => Err(BridgeConditionalDenial::new(
            missing,
            "required provider is absent",
        )),
        (false, true) => Err(BridgeConditionalDenial::new(
            extra,
            "provider was registered for a declaration that does not require it",
        )),
        _ => Ok(()),
    }
}

pub(super) const PROVIDER_DIMENSION_CHECK_COUNT: usize = 7;

const PROVIDER_ROLE_DENIALS: [(BridgeConditionalDenialKind, BridgeConditionalDenialKind);
    PROVIDER_DIMENSION_CHECK_COUNT] = [
    (
        BridgeConditionalDenialKind::MissingConditionProvider,
        BridgeConditionalDenialKind::ExtraConditionProvider,
    ),
    (
        BridgeConditionalDenialKind::MissingDependencyComparator,
        BridgeConditionalDenialKind::ExtraDependencyComparator,
    ),
    (
        BridgeConditionalDenialKind::MissingOutputComparator,
        BridgeConditionalDenialKind::ExtraOutputComparator,
    ),
    (
        BridgeConditionalDenialKind::MissingReuseComparator,
        BridgeConditionalDenialKind::ExtraReuseComparator,
    ),
    (
        BridgeConditionalDenialKind::MissingTriggerProvider,
        BridgeConditionalDenialKind::ExtraTriggerProvider,
    ),
    (
        BridgeConditionalDenialKind::MissingWakeProvider,
        BridgeConditionalDenialKind::ExtraWakeProvider,
    ),
    (
        BridgeConditionalDenialKind::MissingComputeProvider,
        BridgeConditionalDenialKind::ExtraComputeProvider,
    ),
];
