use super::{checkpoint_complete, recovery, runtime_rebuild, RecoveredRelationalBranchBasis};
use crate::branch::AdmittedRelationalBranchBasis;
use crate::history::data::BranchId;

use crate::runtime::RelationalRuntime;

/// The exclusive recovery lane.
///
/// Recovery rebuilds a whole runtime from durable evidence and then installs it
/// in place of the live one, so it is the single durability operation that
/// cannot run against a shared borrow. It is a separate wrapper rather than a
/// method on the shared authority so the exclusive requirement is visible at
/// every call site instead of infecting ordinary durability work.
pub struct DurabilityRecoveryAuthority<'runtime> {
    pub(super) runtime: &'runtime mut RelationalRuntime,
}

impl<'runtime> DurabilityRecoveryAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime mut RelationalRuntime) -> Self {
        Self { runtime }
    }
}

/// Linear evidence that this exact Relational runtime and its captured branch
/// images were reconstructed through an admitted Native recovery plan. Durable
/// backing of each canonical effect is qualified separately from that plan's
/// Memory or Journal residency. The value is
/// owner-issued and deliberately non-serializable, so descriptive checkpoint
/// presence or later branch movement cannot impersonate successful recovery.
#[derive(Debug)]
pub struct RecoveredRelationalRuntimeAuthority {
    pub(super) interim_bootstrap_phase:
        Option<checkpoint_complete::interim::bootstrap::BootstrapPhaseOwner>,
    pub(super) home_catalog_readmission: std::sync::Mutex<()>,
    pub(super) home_program_predecessor:
        Option<crate::runtime::RelationalHomeProgramPredecessorCatalog>,
    pub(super) runtime_instance_id: u64,
    pub(super) recovered_roots:
        std::sync::Arc<runtime_rebuild::root_inventory::RecoveredRootInventory>,
    pub(super) recovered_branch_images: std::collections::BTreeMap<
        BranchId,
        (
            crate::branch::RelationalBranchReferenceObservation,
            crate::branch::RelationalBranchVersion,
        ),
    >,
    pub(super) codec_transition: Option<recovery::codec_transition::RecoveredCodecTransitionState>,
    pub(super) performed_codec_retirement:
        Option<crate::history::data::OwnedCodecRetirementClosure>,
}

impl RecoveredRelationalRuntimeAuthority {
    /// The original Home acknowledges actual World reconciliation, releasing
    /// preparation pins only. A foreign or stale completion changes nothing.
    pub fn complete_home_world_selection(
        &self,
        completed: worth_proof::CompletedHomeWorldSelection,
    ) -> Result<(), worth_proof::HomeWorldSelectionAdmissionDenial> {
        self.recovered_roots
            .complete_world_selection_handoff(completed)
    }

    /// Whether native recovery retained a distinct codec target for adoption.
    pub fn codec_transition_pending(&self) -> bool {
        self.codec_transition.is_some()
    }

    /// Observe the exact closure already staged by native codec admission.
    /// This cannot grant retirement permission to an unstaged record.
    pub fn codec_transition_retires(&self, entity: crate::identity::data::EntityId) -> bool {
        self.codec_transition
            .as_ref()
            .and_then(|state| state.retirement.as_ref())
            .or(self.performed_codec_retirement.as_ref())
            .is_some_and(|closure| closure.entities.contains(&entity))
    }

    /// Observe whether native codec admission already retires this exact relation.
    /// This cannot add a relation to the staged or performed closure.
    pub fn codec_transition_retires_relation(
        &self,
        relation: crate::identity::data::RelationId,
    ) -> bool {
        self.codec_transition
            .as_ref()
            .and_then(|state| state.retirement.as_ref())
            .or(self.performed_codec_retirement.as_ref())
            .is_some_and(|closure| closure.relations.contains(&relation))
    }

    /// Native performed output origins retained by exact retirement admission.
    /// Callers may compare these descriptions to their installed producer
    /// roster; descriptions alone cannot add a record to the native closure.
    pub fn codec_transition_output_origins(
        &self,
    ) -> impl Iterator<
        Item = (
            crate::identity::data::EntityId,
            crate::identity::data::KindId,
            &str,
            &[u8; 32],
        ),
    > {
        self.codec_transition
            .as_ref()
            .and_then(|state| state.retirement.as_ref())
            .or(self.performed_codec_retirement.as_ref())
            .into_iter()
            .flat_map(|closure| closure.producers())
    }

    /// Genuine native birth and the latest performed publication's posture.
    /// These descriptions grant neither currentness nor additional retirement.
    pub fn codec_transition_output_births(
        &self,
    ) -> impl Iterator<
        Item = (
            crate::identity::data::EntityId,
            crate::identity::data::KindId,
            &str,
            &[u8; 32],
            crate::mvcc::RequiredOutputRolePosture,
            crate::mvcc::RequiredOutputRolePosture,
        ),
    > {
        self.codec_transition
            .as_ref()
            .and_then(|state| state.retirement.as_ref())
            .or(self.performed_codec_retirement.as_ref())
            .into_iter()
            .flat_map(|closure| closure.births())
    }

    pub fn admit_basis(
        self,
        basis: AdmittedRelationalBranchBasis,
    ) -> Result<RecoveredRelationalBranchBasis, AdmittedRelationalBranchBasis> {
        let matches = self.owns_image(basis.descriptor());
        if let Some(phase) = &self.interim_bootstrap_phase {
            phase.finish_capture(matches);
        }
        if matches {
            Ok(RecoveredRelationalBranchBasis { basis })
        } else {
            Err(basis)
        }
    }

    pub(super) fn owns_image(
        &self,
        descriptor: &crate::branch::RelationalBranchBasisDescriptor,
    ) -> bool {
        let matches_recovered_image = self
            .recovered_branch_images
            .get(descriptor.branch_id())
            .is_some_and(|(reference, truth_version)| {
                reference == descriptor.reference() && *truth_version == descriptor.truth_version()
            });
        descriptor.runtime_instance_id() == self.runtime_instance_id && matches_recovered_image
    }
}
