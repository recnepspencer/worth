use super::{RefusedRelationalRecordedComponent, RelationalRecordedComponentAdmissionCause};
use crate::branch::recorded_basis::{
    RecordedRelationalComponentReference as Recorded, RecordedRelationalComponentTarget as Target,
};
use crate::durability::authority::recovered_runtime::RecoveredRelationalRuntimeAuthority;
use crate::durability::authority::RecoveredRelationalBranchBasis;
use crate::runtime::RelationalRuntime;

impl RecoveredRelationalRuntimeAuthority {
    /// Admit a recorded Empty or Canonical component from this recovery's
    /// actual image. Caller data cannot choose a later root or live identity.
    pub fn admit_recorded_component(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        recorded: &Recorded,
    ) -> Result<RecoveredRelationalBranchBasis, RefusedRelationalRecordedComponent> {
        use RelationalRecordedComponentAdmissionCause as Cause;
        let refused = |cause| RefusedRelationalRecordedComponent {
            cause,
            acquired: None,
        };
        crate::execution::check_request(execution)
            .map_err(|cause| refused(Cause::Request(cause)))?;
        if self.runtime_instance_id != runtime.runtime_instance_id() {
            return Err(refused(Cause::ForeignRecoveryRuntime));
        }
        let _operation = runtime
            .owner_binding()
            .admit()
            .ok_or_else(|| refused(Cause::OwnerUnavailable))?;
        let (root, descriptor) = self
            .recovered_roots
            .exact_image(execution, runtime, recorded)
            .map_err(|cause| refused(Cause::Source(cause)))?
            .ok_or_else(|| refused(Cause::MissingRecoveredImage))?;
        match recorded.target() {
            Target::Empty
                if matches!(
                    descriptor.reference().target(),
                    worth_foundational::FoundationalBranchTarget::Empty
                ) && root.id() == 0
                    && root.descriptor().is_none() => {}
            Target::Canonical(target) => {
                let evidence = root
                    .commit_evidence()
                    .ok_or_else(|| refused(Cause::RecordedTargetMismatch))?;
                if evidence.receipt() != target.receipt()
                    || root.descriptor() != Some(target.roots())
                {
                    return Err(refused(Cause::RecordedTargetMismatch));
                }
                let artifact = runtime
                    .history
                    .commit_artifact(target.receipt().commit_id)
                    .ok_or_else(|| refused(Cause::MissingRecoveredImage))?;
                let (entry, source) = artifact
                    .envelope_residency()
                    .source_placement()
                    .ok_or_else(|| refused(Cause::MemoryOnly))?;
                if entry.commit != *target.receipt()
                    || entry.position != target.position()
                    || &entry.payload_digest != target.artifact_digest()
                    || &entry.root != target.roots()
                {
                    return Err(refused(Cause::RecordedTargetMismatch));
                }
                match evidence {
                    crate::branch::RelationalRootCommitEvidence::Original(original) => {
                        let admitted = crate::durability::history_source_readmission::readmit_original_positioned(
                            &artifact.envelope_residency(),
                            execution,
                        )
                            .map_err(|cause| refused(Cause::Source(cause)))?;
                        if !crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::matches_positioned_payload_for_request(
                            &admitted, original.envelope(), execution,
                        ).map_err(|cause| refused(Cause::Source(cause)))? {
                            return Err(refused(Cause::RecordedTargetMismatch));
                        }
                    }
                    crate::branch::RelationalRootCommitEvidence::Checkpoint(commit) => {
                        if !runtime
                            .durability
                            .checkpoint_selection()
                            .matches_admitted_source(commit, &source)
                            || commit.index() != entry.as_ref()
                        {
                            return Err(refused(Cause::RecordedTargetMismatch));
                        }
                    }
                }
            }
            _ => return Err(refused(Cause::RecordedTargetMismatch)),
        }
        let identity = runtime
            .branch_identity(descriptor.branch_id())
            .map_err(|_| refused(Cause::MissingRecoveredImage))?;
        let cell = runtime
            .history
            .branch_cell(descriptor.branch_id())
            .ok_or_else(|| refused(Cause::MissingRecoveredImage))?;
        crate::execution::check_request(execution)
            .map_err(|cause| refused(Cause::Request(cause)))?;
        let binding = cell
            .head_retention()
            .binding()
            .map_err(|_| refused(Cause::MissingRecoveredImage))?;
        let basis = crate::branch::issue_admitted_relational_branch_basis_with_retention(
            descriptor,
            identity,
            root,
            cell.publication_cell(),
            &binding,
        )
        .map_err(|cause| refused(Cause::Basis(cause)))?;
        match cell.register_basis(basis.clone()) {
            Ok(basis) => Ok(RecoveredRelationalBranchBasis { basis }),
            Err(cause) => Err(RefusedRelationalRecordedComponent {
                cause: Cause::Basis(cause),
                acquired: Some(basis),
            }),
        }
    }
}
