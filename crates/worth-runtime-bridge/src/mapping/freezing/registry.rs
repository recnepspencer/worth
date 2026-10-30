use super::validation::{validate_registration_set, validate_registration_values};
use super::*;
use sha2::{Digest, Sha256};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrozenBridgeMappingRegistration {
    registration: BridgeMappingRegistration,
    registration_identity: BridgeFrozenMappingRegistrationIdentity,
}

impl FrozenBridgeMappingRegistration {
    fn new(registration: BridgeMappingRegistration) -> Self {
        let registration_identity = frozen_registration_identity(&registration);
        Self {
            registration,
            registration_identity,
        }
    }

    pub fn registration_identity(&self) -> &BridgeFrozenMappingRegistrationIdentity {
        &self.registration_identity
    }

    pub fn mapping_id(&self) -> &crate::mapping::registration::BridgeMappingId {
        self.registration.mapping_id()
    }

    pub fn truth_scope(&self) -> &crate::mapping::registration::TruthPatchScope {
        self.registration.truth_scope()
    }

    pub fn snapshot_read_contract(&self) -> &crate::snapshot::SnapshotReadContract {
        self.registration.snapshot_read_contract()
    }

    pub fn signal_scope(&self) -> &crate::mapping::registration::SignalInvalidationScope {
        self.registration.signal_scope()
    }

    pub fn routing_mode(&self) -> crate::mapping::registration::CoarseRoutingMode {
        self.registration.routing_mode()
    }

    pub fn widening_class(&self) -> Option<BridgeMappingWideningClass> {
        self.registration.truth_scope().widening_class()
    }
}

fn frozen_registration_identity(
    registration: &BridgeMappingRegistration,
) -> BridgeFrozenMappingRegistrationIdentity {
    BridgeFrozenMappingRegistrationIdentity::admit_bridge_owned(digest_string(
        "frozen-mapping-registration",
        &frozen_registration_identity_basis(registration),
    ))
}

fn frozen_registration_identity_basis(registration: &BridgeMappingRegistration) -> String {
    format!(
        "frozen-mapping-registration|mapping={}|truth-scope={}|snapshot-read-contract={}|signal-scope={}|routing-mode={}",
        registration.mapping_id().as_str(),
        truth_scope_canonical_basis(registration.truth_scope()),
        registration.snapshot_read_contract().canonical_basis(),
        registration.signal_scope().as_str(),
        routing_mode_label(registration.routing_mode())
    )
}

fn truth_scope_canonical_basis(scope: &TruthPatchScope) -> String {
    format!(
        "truth-patch-scope|entity={}|aspect={}|target={}",
        mapping_selector_basis(scope.entity_selector()),
        aspect_selector_basis(scope.aspect_selector()),
        scope.target_selector().canonical_basis()
    )
}

fn mapping_selector_basis(selector: &MappingSelector) -> Arc<str> {
    match selector {
        MappingSelector::Any => Arc::from("mapping-selector|kind=any"),
        MappingSelector::Exact(value) => {
            Arc::from(format!("mapping-selector|kind=exact|value={value}"))
        }
    }
}

fn aspect_selector_basis(selector: &AspectKeySelector) -> Arc<str> {
    match selector {
        AspectKeySelector::Any => Arc::from("aspect-selector|kind=any"),
        AspectKeySelector::Exact(aspect_key) => Arc::from(format!(
            "aspect-selector|kind=exact|aspect={}",
            aspect_key.as_str()
        )),
    }
}

fn routing_mode_label(mode: CoarseRoutingMode) -> &'static str {
    match mode {
        CoarseRoutingMode::Direct => "direct",
    }
}

fn digest_string(kind: &str, basis: &str) -> Arc<str> {
    let digest = Sha256::digest(basis.as_bytes());
    format!("{kind}:sha256:{digest:x}").into()
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FrozenMappingRegistry {
    pub(crate) registrations: Vec<FrozenBridgeMappingRegistration>,
}

impl FrozenMappingRegistry {
    pub(crate) fn freeze(
        mut registrations: Vec<BridgeMappingRegistration>,
    ) -> Result<Self, BridgeBuildError> {
        if registrations.is_empty() {
            return Err(BridgeBuildError::new(
                BridgeBuildErrorKind::MissingMappingRegistrations,
                "RuntimeBridgeBuilder requires at least one bridge mapping registration before build.",
            ));
        }

        registrations.sort_by(canonical_registration_order);
        validate_registration_values(&registrations)?;
        validate_registration_set(&registrations)?;

        let registrations = registrations
            .into_iter()
            .map(FrozenBridgeMappingRegistration::new)
            .collect::<Vec<_>>();
        Ok(Self { registrations })
    }

    pub(crate) fn registrations(&self) -> &[FrozenBridgeMappingRegistration] {
        &self.registrations
    }
}

fn canonical_registration_order(
    left: &BridgeMappingRegistration,
    right: &BridgeMappingRegistration,
) -> std::cmp::Ordering {
    right
        .truth_scope()
        .specificity_rank()
        .cmp(&left.truth_scope().specificity_rank())
        .then_with(|| left.truth_scope().cmp(right.truth_scope()))
        .then_with(|| {
            left.snapshot_read_contract()
                .canonical_basis()
                .cmp(right.snapshot_read_contract().canonical_basis())
        })
        .then_with(|| left.signal_scope().cmp(right.signal_scope()))
        .then_with(|| left.routing_mode().cmp(&right.routing_mode()))
        .then_with(|| left.mapping_id().cmp(right.mapping_id()))
}
