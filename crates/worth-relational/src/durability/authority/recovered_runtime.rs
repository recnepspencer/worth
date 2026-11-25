use super::{recovery, runtime_rebuild, HomeReadmissionMetadataPhase, RecoveredRelationalBranchBasis};
use crate::branch::AdmittedRelationalBranchBasis;
use crate::history::data::BranchId;
use worth_relational_durability::internal_port::durability::authority::RecoveredRelationalBranchBasisOwnerPort as _;

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
    pub(super) home_catalog_readmission: std::sync::Mutex<()>,
    pub(super) home_program_predecessor:
        Option<crate::runtime::RelationalHomeProgramPredecessorCatalog>,
    pub(super) runtime_instance_id: u64,
    pub(super) recovered_roots:
        std::sync::Arc<runtime_rebuild::root_inventory::RecoveredRootInventory>,
    pub(super) home_readmission_phase: std::sync::Arc<HomeReadmissionMetadataPhase>,
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

/// Linear, non-cloneable authority issued at most once from a recovered Native
/// runtime during its active Home readmission phase (before service begins) to
/// project owner-admitted internal metadata records on a recovered branch.
///
/// Dropping this authority, completing Home readmission, or committing any
/// post-recovery transaction closes the phase for this authority and any view.
///
/// Ordinary callers cannot construct this witness:
/// ```compile_fail,E0451
/// use worth_relational::facade::durability::HomeReadmissionMetadataAuthority;
/// let _ = HomeReadmissionMetadataAuthority { phase: todo!() };
/// ```
///
/// Nor clone it to survive service handoff:
/// ```compile_fail,E0277
/// use worth_relational::facade::durability::HomeReadmissionMetadataAuthority;
/// fn require_clone<T: Clone>() {}
/// require_clone::<HomeReadmissionMetadataAuthority>();
/// ```
#[derive(Debug)]
pub struct HomeReadmissionMetadataAuthority {
    phase: std::sync::Arc<HomeReadmissionMetadataPhase>,
}

impl Drop for HomeReadmissionMetadataAuthority {
    fn drop(&mut self) {
        self.phase.close();
    }
}

impl HomeReadmissionMetadataAuthority {
    pub(crate) fn runtime_instance_id(&self) -> u64 {
        self.phase.runtime_instance_id()
    }

    pub(crate) fn recovered_latest_commit(&self) -> Option<crate::history::data::CommitId> {
        self.phase.recovered_latest_commit()
    }

    pub(crate) fn is_active(&self) -> bool {
        self.phase.is_active()
    }

    pub(crate) fn allows_entity_kind(&self, kind_id: crate::identity::data::KindId) -> bool {
        self.phase.allows_entity_kind(kind_id)
    }

    pub(crate) fn allows_relation_kind(&self, kind_id: crate::identity::data::KindId) -> bool {
        self.phase.allows_relation_kind(kind_id)
    }
}

impl RecoveredRelationalRuntimeAuthority {
    /// Issue the single linear Home-readmission metadata authority bound to
    /// this recovered runtime's active pre-service readmission phase and its
    /// sealed owner-admitted metadata kinds.
    ///
    /// Fails with `OwnerUnavailable` if already issued, if `finish_home_readmission`
    /// has run, or if the runtime has already published a post-recovery commit.
    pub fn begin_home_readmission_metadata(
        &self,
    ) -> Result<
        HomeReadmissionMetadataAuthority,
        crate::branch::basis_denial::RelationalBranchBasisDenial,
    > {
        use crate::branch::basis_denial::RelationalBranchBasisDenial as Denial;
        if !self.home_readmission_phase.is_active() || !self.home_readmission_phase.issue_once() {
            return Err(Denial::OwnerUnavailable);
        }
        if !self.home_readmission_phase.is_active() {
            return Err(Denial::OwnerUnavailable);
        }
        Ok(HomeReadmissionMetadataAuthority {
            phase: std::sync::Arc::clone(&self.home_readmission_phase),
        })
    }

    /// Irreversibly end the active Home readmission phase at service handoff,
    /// expiring any issued authority or metadata view and preventing re-issuance.
    pub fn finish_home_readmission(&self) {
        self.home_readmission_phase.close();
    }
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
        if self.owns_image(basis.descriptor()) {
            Ok(RecoveredRelationalBranchBasis::from_readmission(basis))
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
