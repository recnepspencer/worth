use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;

use super::{
    WorthQueryAdmittedCollectionWindow, WorthQueryCollectionCursor, WorthQueryCollectionRowHandle,
    WorthQueryCollectionWindowCounters, WorthQueryCollectionWindowDenial,
};
use crate::domain_installation::WorthQueryOperationResultState;

pub type WorthQueryCollectionWindowOutcome = TransitionOutcome<
    WorthQueryBoundCollectionWindow,
    WorthQueryCollectionWindowDenial,
    std::convert::Infallible,
    WorthQueryCollectionWindowDenial,
    WorthQueryCollectionWindowDenial,
>;

pub type WorthQueryCollectionWindowAdmissionOutcome = TransitionOutcome<
    WorthQueryAdmittedCollectionWindow,
    WorthQueryCollectionWindowDenial,
    std::convert::Infallible,
    WorthQueryCollectionWindowDenial,
    WorthQueryCollectionWindowDenial,
>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryCollectionContinuation {
    Complete,
    SnapshotMore(WorthQueryCollectionCursor),
    LiveMore(WorthQueryCollectionCursor),
}

impl WorthQueryCollectionContinuation {
    pub fn identity_evidence(
        &self,
    ) -> Result<Option<crate::WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
        let (posture, cursor) = match self {
            Self::Complete => return Ok(None),
            Self::SnapshotMore(cursor) => ("snapshot-more", cursor),
            Self::LiveMore(cursor) => ("live-more", cursor),
        };
        Ok(Some(
            crate::WorthQueryEvidenceIdentity::compose(
                crate::WorthQueryEvidenceScope::ProjectionConsumptionIdentity,
            )
            .field_shape(
                crate::WorthQueryEvidenceTag::new("collection"),
                "continuation",
            )
            .field_shape(crate::WorthQueryEvidenceTag::new("posture"), posture)
            .field_evidence_identity(
                crate::WorthQueryEvidenceTag::new("cursor"),
                &cursor.identity_evidence(),
            )
            .seal()?,
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryCollectionWindowWarning {
    ExecutionWarningsPresent { count: usize },
    ProjectionWarningsPresent,
    MountingBudgetClamped,
}

pub struct WorthQueryBoundCollectionWindow {
    pub(crate) capability_identity: u64,
    pub(crate) capability_generation:
        crate::domain_installation::WorthQueryBoundCapabilityGeneration,
    pub(crate) source_identity: String,
    pub(crate) binding_identity: String,
    pub(crate) result_shape_identity: String,
    pub(crate) collection_delivery_contract_identity: String,
    pub(crate) window_contract_identity: String,
    pub(crate) basis_identity: String,
    pub(crate) ordering_identity: String,
    pub(crate) admitted_width: usize,
    cursor: WorthQueryCollectionCursor,
    rows: Vec<WorthQueryCollectionRowHandle>,
    continuation: WorthQueryCollectionContinuation,
    result_state: WorthQueryOperationResultState,
    warnings: Vec<WorthQueryCollectionWindowWarning>,
    counters: WorthQueryCollectionWindowCounters,
}

pub(crate) struct WorthQueryCollectionWindowParts {
    pub capability_identity: u64,
    pub capability_generation: crate::domain_installation::WorthQueryBoundCapabilityGeneration,
    pub source_identity: String,
    pub binding_identity: String,
    pub result_shape_identity: String,
    pub collection_delivery_contract_identity: String,
    pub window_contract_identity: String,
    pub basis_identity: String,
    pub ordering_identity: String,
    pub admitted_width: usize,
    pub cursor: WorthQueryCollectionCursor,
    pub rows: Vec<WorthQueryCollectionRowHandle>,
    pub continuation: WorthQueryCollectionContinuation,
    pub result_state: WorthQueryOperationResultState,
    pub warnings: Vec<WorthQueryCollectionWindowWarning>,
    pub counters: WorthQueryCollectionWindowCounters,
}

impl WorthQueryBoundCollectionWindow {
    pub(crate) fn from_parts(parts: WorthQueryCollectionWindowParts) -> Self {
        Self {
            capability_identity: parts.capability_identity,
            capability_generation: parts.capability_generation,
            source_identity: parts.source_identity,
            binding_identity: parts.binding_identity,
            result_shape_identity: parts.result_shape_identity,
            collection_delivery_contract_identity: parts.collection_delivery_contract_identity,
            window_contract_identity: parts.window_contract_identity,
            basis_identity: parts.basis_identity,
            ordering_identity: parts.ordering_identity,
            admitted_width: parts.admitted_width,
            cursor: parts.cursor,
            rows: parts.rows,
            continuation: parts.continuation,
            result_state: parts.result_state,
            warnings: parts.warnings,
            counters: parts.counters,
        }
    }

    pub fn cursor(&self) -> &WorthQueryCollectionCursor {
        &self.cursor
    }

    pub fn rows(&self) -> &[WorthQueryCollectionRowHandle] {
        &self.rows
    }

    pub fn continuation(&self) -> &WorthQueryCollectionContinuation {
        &self.continuation
    }

    pub const fn result_state(&self) -> WorthQueryOperationResultState {
        self.result_state
    }

    pub fn warnings(&self) -> &[WorthQueryCollectionWindowWarning] {
        &self.warnings
    }

    pub const fn counters(&self) -> WorthQueryCollectionWindowCounters {
        self.counters
    }

    pub(crate) fn admitted_width(&self) -> usize {
        self.admitted_width
    }

    pub(crate) fn targetized(
        &self,
        capability_identity: u64,
        capability_generation: crate::domain_installation::WorthQueryBoundCapabilityGeneration,
    ) -> Self {
        let continuation = match &self.continuation {
            WorthQueryCollectionContinuation::Complete => {
                WorthQueryCollectionContinuation::Complete
            }
            WorthQueryCollectionContinuation::SnapshotMore(cursor) => {
                WorthQueryCollectionContinuation::SnapshotMore(
                    cursor.rebind(capability_identity, capability_generation),
                )
            }
            WorthQueryCollectionContinuation::LiveMore(cursor) => {
                WorthQueryCollectionContinuation::LiveMore(
                    cursor.rebind(capability_identity, capability_generation),
                )
            }
        };
        Self {
            capability_identity,
            capability_generation,
            source_identity: self.source_identity.clone(),
            binding_identity: self.binding_identity.clone(),
            result_shape_identity: self.result_shape_identity.clone(),
            collection_delivery_contract_identity: self
                .collection_delivery_contract_identity
                .clone(),
            window_contract_identity: self.window_contract_identity.clone(),
            basis_identity: self.basis_identity.clone(),
            ordering_identity: self.ordering_identity.clone(),
            admitted_width: self.admitted_width,
            cursor: self
                .cursor
                .rebind(capability_identity, capability_generation),
            rows: self
                .rows
                .iter()
                .map(|row| row.rebind(capability_identity, capability_generation))
                .collect(),
            continuation,
            result_state: self.result_state,
            warnings: self.warnings.clone(),
            counters: self.counters,
        }
    }
}
