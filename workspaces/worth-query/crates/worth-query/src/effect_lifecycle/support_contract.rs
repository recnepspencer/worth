use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::support_matrix::{EffectSupportCause, EffectSupportPosture};
use super::taxonomy::{DeniedEffectEligibilityKind, EffectFamily};

const EFFECT_LIFECYCLE_IDENTITY_SCOPE: WorthQueryEvidenceScope =
    WorthQueryEvidenceScope::WorkflowMutationLowering;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectDeferredNeighborFamily {
    StoreBackedExecutionParity,
    DurableReplayAndRestartStableEnvelope,
}

impl EffectDeferredNeighborFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StoreBackedExecutionParity => "store_backed_execution_parity",
            Self::DurableReplayAndRestartStableEnvelope => {
                "durable_replay_and_restart_stable_envelope"
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectDeferredResiduePosture {
    ZeroOperationalResidue,
}

impl EffectDeferredResiduePosture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ZeroOperationalResidue => "zero_operational_residue",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectDeferredSupportContract {
    neighbor_family: EffectDeferredNeighborFamily,
    denial_kind: DeniedEffectEligibilityKind,
    residue_posture: EffectDeferredResiduePosture,
    contract_identity: WorthQueryEvidenceIdentity,
}

impl EffectDeferredSupportContract {
    fn new(
        neighbor_family: EffectDeferredNeighborFamily,
        denial_kind: DeniedEffectEligibilityKind,
    ) -> Self {
        let residue_posture = EffectDeferredResiduePosture::ZeroOperationalResidue;
        let contract_identity =
            WorthQueryEvidenceIdentity::compose(EFFECT_LIFECYCLE_IDENTITY_SCOPE)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_deferred_support_contract_v1",
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("neighbor"),
                    neighbor_family.as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("denial_kind"),
                    denial_kind.as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("residue"),
                    residue_posture.as_str(),
                )
                .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens");
        Self {
            neighbor_family,
            denial_kind,
            residue_posture,
            contract_identity,
        }
    }

    pub fn neighbor_family(&self) -> EffectDeferredNeighborFamily {
        self.neighbor_family
    }

    pub fn denial_kind(&self) -> DeniedEffectEligibilityKind {
        self.denial_kind
    }

    pub fn residue_posture(&self) -> EffectDeferredResiduePosture {
        self.residue_posture
    }

    pub fn contract_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.contract_identity
    }

    pub fn contract_for_reporting(&self) -> &str {
        self.contract_identity.as_str()
    }

    pub fn leaves_zero_operational_residue(&self) -> bool {
        self.residue_posture == EffectDeferredResiduePosture::ZeroOperationalResidue
    }
}

const EMPTY_DENIAL_KINDS: &[DeniedEffectEligibilityKind] = &[];
const UNSUPPORTED_DENIAL_KINDS: &[DeniedEffectEligibilityKind] =
    &[DeniedEffectEligibilityKind::UnsupportedForBasisFamily];
const BRANCH_AUTHORITY_DENIAL_KINDS: &[DeniedEffectEligibilityKind] =
    &[DeniedEffectEligibilityKind::BranchAuthorityRequired];
const PREVIEW_REBIND_DENIAL_KINDS: &[DeniedEffectEligibilityKind] =
    &[DeniedEffectEligibilityKind::PreviewRebindRequired];
const STORE_BACKED_DEFERRED_DENIAL_KINDS: &[DeniedEffectEligibilityKind] =
    &[DeniedEffectEligibilityKind::StoreBackedExecutionDeferred];
const DURABLE_REPLAY_DEFERRED_DENIAL_KINDS: &[DeniedEffectEligibilityKind] =
    &[DeniedEffectEligibilityKind::DurableReplayDeferred];

const EMPTY_DEFERRED_NEIGHBORS: &[EffectDeferredNeighborFamily] = &[];
const WRITEBACK_DEFERRED_NEIGHBORS: &[EffectDeferredNeighborFamily] = &[
    EffectDeferredNeighborFamily::StoreBackedExecutionParity,
    EffectDeferredNeighborFamily::DurableReplayAndRestartStableEnvelope,
];

pub(crate) fn support_denial_kinds(
    posture: EffectSupportPosture,
    cause: EffectSupportCause,
) -> &'static [DeniedEffectEligibilityKind] {
    match (posture, cause) {
        (EffectSupportPosture::Unsupported, EffectSupportCause::UnsupportedForBasisFamily) => {
            UNSUPPORTED_DENIAL_KINDS
        }
        (EffectSupportPosture::Denied, EffectSupportCause::BranchAuthorityRequired) => {
            BRANCH_AUTHORITY_DENIAL_KINDS
        }
        (EffectSupportPosture::RebindRequired, EffectSupportCause::PreviewRebindRequired) => {
            PREVIEW_REBIND_DENIAL_KINDS
        }
        (EffectSupportPosture::Deferred, EffectSupportCause::StoreBackedExecutionDeferred) => {
            STORE_BACKED_DEFERRED_DENIAL_KINDS
        }
        (EffectSupportPosture::Deferred, EffectSupportCause::DurableReplayDeferred) => {
            DURABLE_REPLAY_DEFERRED_DENIAL_KINDS
        }
        _ => EMPTY_DENIAL_KINDS,
    }
}

pub(crate) fn support_deferred_neighbors(
    effect_family: EffectFamily,
) -> &'static [EffectDeferredNeighborFamily] {
    match effect_family {
        EffectFamily::Writeback => WRITEBACK_DEFERRED_NEIGHBORS,
        EffectFamily::Mutation | EffectFamily::Merge => EMPTY_DEFERRED_NEIGHBORS,
    }
}

pub(crate) fn deferred_support_contract(
    cause: EffectSupportCause,
) -> Option<EffectDeferredSupportContract> {
    match cause {
        EffectSupportCause::StoreBackedExecutionDeferred => {
            Some(EffectDeferredSupportContract::new(
                EffectDeferredNeighborFamily::StoreBackedExecutionParity,
                DeniedEffectEligibilityKind::StoreBackedExecutionDeferred,
            ))
        }
        EffectSupportCause::DurableReplayDeferred => Some(EffectDeferredSupportContract::new(
            EffectDeferredNeighborFamily::DurableReplayAndRestartStableEnvelope,
            DeniedEffectEligibilityKind::DurableReplayDeferred,
        )),
        _ => None,
    }
}
