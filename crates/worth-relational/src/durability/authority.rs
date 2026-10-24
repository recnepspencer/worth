pub(crate) use recovery::{AdmittedRecoveredCodecPublication, RecoveredCodecPublicationCustody};
mod authority_continuity;
mod checkpoint_capture;
mod checkpoint_image;
mod checkpoint_transition;
mod checkpointing;
mod diagnostics;
mod recovery;
mod runtime_rebuild;
pub(crate) use runtime_rebuild::replay_commit_stop;

use crate::branch::AdmittedRelationalBranchBasis;
use crate::history::data::BranchId;
use crate::runtime::RelationalRuntime;

pub use checkpoint_transition::{
    RecoveredCheckpointTransition, RecoveredCheckpointTransitionDenial,
    RecoveredCheckpointTransitionError, RefusedRecoveredCheckpointTransition,
};

pub struct DurabilityAuthority<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

impl<'runtime> DurabilityAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}

/// The exclusive recovery lane.
///
/// Recovery rebuilds a whole runtime from durable evidence and then installs it
/// in place of the live one, so it is the single durability operation that
/// cannot run against a shared borrow. It is a separate wrapper rather than a
/// method on the shared authority so the exclusive requirement is visible at
/// every call site instead of infecting ordinary durability work.
pub struct DurabilityRecoveryAuthority<'runtime> {
    runtime: &'runtime mut RelationalRuntime,
}

/// Linear evidence that this exact Relational runtime and its captured branch
/// images were rebuilt from a verified native checkpoint. The value is
/// owner-issued and deliberately non-serializable, so descriptive checkpoint
/// presence or later branch movement cannot impersonate successful recovery.
#[derive(Debug)]
pub struct RecoveredRelationalRuntimeAuthority {
    runtime_instance_id: u64,
    recovered_branch_images: std::collections::BTreeMap<
        BranchId,
        (
            crate::branch::RelationalBranchReferenceObservation,
            crate::branch::RelationalBranchVersion,
        ),
    >,
    codec_transition: Option<recovery::codec_transition::RecoveredCodecTransitionState>,
    performed_codec_retirement: Option<crate::history::data::OwnedCodecRetirementClosure>,
}

impl RecoveredRelationalRuntimeAuthority {
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
            Ok(RecoveredRelationalBranchBasis { basis })
        } else {
            Err(basis)
        }
    }

    fn owns_image(&self, descriptor: &crate::branch::RelationalBranchBasisDescriptor) -> bool {
        let matches_recovered_image = self
            .recovered_branch_images
            .get(descriptor.branch_id())
            .is_some_and(|(reference, truth_version)| {
                reference == descriptor.reference() && *truth_version == descriptor.truth_version()
            });
        descriptor.runtime_instance_id() == self.runtime_instance_id && matches_recovered_image
    }
}

/// An operational branch basis proven to belong to a successfully recovered
/// Relational runtime. Only recovery authority can construct this wrapper.
#[derive(Debug)]
pub struct RecoveredRelationalBranchBasis {
    basis: AdmittedRelationalBranchBasis,
}

impl RecoveredRelationalBranchBasis {
    pub fn into_basis(self) -> AdmittedRelationalBranchBasis {
        self.basis
    }
}

impl<'runtime> DurabilityRecoveryAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime mut RelationalRuntime) -> Self {
        Self { runtime }
    }
}

impl RelationalRuntime {
    pub fn durability_authority(&self) -> DurabilityAuthority<'_> {
        DurabilityAuthority::new(self)
    }

    pub fn durability_recovery(&mut self) -> DurabilityRecoveryAuthority<'_> {
        DurabilityRecoveryAuthority::new(self)
    }
}
