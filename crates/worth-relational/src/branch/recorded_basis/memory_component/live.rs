//! Bind memory components to their original live runtime and source.
use super::*;
use crate::runtime::RelationalRuntime;
use worth_foundational::FoundationalBranchTarget;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
use worth_relational_branch::internal_port::branch::recorded_basis::copying::RecordedRelationalComponentReferenceOwnerPort as _;
use worth_relational_branch::internal_port::branch::recorded_basis::memory_component::assemble_observed_relational_memory_component;
use worth_relational_branch::internal_port::branch::recorded_basis::memory_component::ObservedRelationalMemoryComponentFieldsPort as _;
use worth_relational_branch::internal_port::branch::recorded_basis::memory_component::ObservedRelationalMemoryComponentOwnerPort as _;
use worth_relational_branch::internal_port::branch::recorded_basis::RecordedRelationalComponentReferenceOwnerPort as _;

impl RelationalRuntime {
    pub(crate) fn observe_memory_component(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<ObservedRelationalMemoryComponent, Refusal> {
        crate::execution::request_custody::check_request(execution).map_err(Refusal::Request)?;
        crate::branch::basis_identity_validation::require_local_branch_identity(
            self.runtime_instance_id(),
            &self.history,
            basis.identity(),
        )
        .map_err(Refusal::Basis)?;
        let descriptor = basis.descriptor();
        if basis.owner_inner().root.canonical_envelope().is_none()
            && let Some(target) = descriptor.reference().target().as_basis()
        {
            let commit_id = crate::history::data::CommitId(target.selected_commit_id());
            let artifact = self
                .history
                .commit_artifact(commit_id)
                .ok_or(Refusal::MissingPerformedCommit { commit_id })?;
            let Some((entry, source)) = artifact.envelope_residency().source_placement() else {
                return Err(Refusal::CanonicalMismatch { commit_id });
            };
            let row = self
                .durability
                .checkpoint_selection()
                .admitted_commit(&entry)
                .map_err(map_interim_admission_refusal)?
                .ok_or(Refusal::CanonicalMismatch { commit_id })?;
            let interim = match row.scope() {
                worth_foundational::facade::CheckpointAdmissionScope::Interim { .. } => true,
                worth_foundational::facade::CheckpointAdmissionScope::Home { .. } => false,
            };
            if !interim
                || !row.links_artifact(&artifact)
                || !self
                    .durability
                    .checkpoint_selection()
                    .matches_admitted_source(&row, &source)
                || row.index().commit.version_id.0 != target.version_id()
                || &row.index().root != target.roots()
            {
                return Err(Refusal::CanonicalMismatch { commit_id });
            }
            let description = RecordedRelationalComponentReference::from_readmitted_source(
                execution, descriptor, &entry,
            )?;
            return ObservedRelationalMemoryComponent::from_interim(
                execution,
                self.services.origin_identity(),
                description,
                Some(row),
            )
            .map_err(map_interim_admission_refusal);
        }
        let bytes = successor::live_bytes(descriptor, successor::parent_count(descriptor))?;
        let funding = successor::fund_and_charge(execution, bytes)?;
        self.live_component(basis, funding)
    }
    /// The Live observation of `basis`, whose copy `funding` already paid.
    pub(super) fn live_component(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        funding: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Result<ObservedRelationalMemoryComponent, Refusal> {
        let descriptor = basis.descriptor();
        let canonical = basis.owner_inner().root.canonical_envelope();
        if descriptor.reference().target().as_basis().is_some() && canonical.is_none() {
            return Err(Refusal::CanonicalMismatch {
                commit_id: crate::history::data::CommitId(
                    descriptor
                        .reference()
                        .target()
                        .as_basis()
                        .unwrap()
                        .selected_commit_id(),
                ),
            });
        }
        Ok(assemble_observed_relational_memory_component(Arc::new(
            MemoryComponentOrigin::Live(MemoryComponentMeaning {
                owner: self.services.origin_identity(),
                descriptor: descriptor.clone(),
                canonical: canonical.map(Arc::downgrade),
                _funding: funding,
            }),
        )))
    }
    pub(crate) fn export_memory_component(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        component: &ObservedRelationalMemoryComponent,
    ) -> Result<RecordedRelationalComponentReference, Refusal> {
        crate::execution::request_custody::check_request(execution).map_err(Refusal::Request)?;
        let origin = self.services.origin_identity();
        let meaning = match component.owner_original() {
            MemoryComponentOrigin::Live(meaning) => meaning,
            MemoryComponentOrigin::Interim(meaning) => {
                return meaning.export(&origin, &self.history)
            }
        };
        super::capture::require_origin(&meaning.owner, &origin)?;
        let descriptor = &meaning.descriptor;
        match descriptor.reference().target() {
            FoundationalBranchTarget::Empty => {
                RecordedRelationalComponentReference::copy_original(execution, descriptor, None)
            }
            FoundationalBranchTarget::Basis(target) => {
                let commit_id = crate::history::data::CommitId(target.selected_commit_id());
                let artifact = self
                    .history
                    .commit_artifact(commit_id)
                    .ok_or(Refusal::MissingPerformedCommit { commit_id })?;
                let cell = artifact.envelope_residency();
                let positioned = cell
                    .demand_positioned(execution, 1 << 30)
                    .map_err(Refusal::Source)?;
                let same_original = meaning.canonical.as_ref().is_some_and(|original| {
                    Weak::ptr_eq(original, &Arc::downgrade(positioned.canonical_arc()))
                });
                if !same_original
                    || positioned.envelope().commit.commit_id != commit_id
                    || positioned.envelope().commit.version_id.0 != target.version_id()
                    || artifact.roots() != target.roots()
                {
                    return Err(Refusal::CanonicalMismatch { commit_id });
                }
                let digest =
                    crate::history::commit::payload_commitment::digest_envelope_for_request(
                        positioned.position(),
                        positioned.envelope(),
                        execution,
                    )
                    .map_err(Refusal::Source)?;
                RecordedRelationalComponentReference::copy_original(
                    execution,
                    descriptor,
                    Some(copying::CanonicalDescription {
                        receipt: &positioned.envelope().commit,
                        position: positioned.position(),
                        artifact_digest: digest,
                        roots: target.roots(),
                    }),
                )
            }
        }
    }
}
