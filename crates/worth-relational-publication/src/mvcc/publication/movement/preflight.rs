//! Select and retain the successor that this prepared movement may install.
use contracts::mvcc::publication::outcome::failure::RelationalPublicationFailureOwnerPort as _;
use std::sync::Arc;
use worth_relational_contracts::internal_port as contracts;

use super::PreparedCanonicalBranchMovement;
use super::RelationalPublicationOutcome;
use contracts::mvcc::publication::outcome::failure::RelationalPublicationFailure;
use contracts::mvcc::publication::outcome::failure::RelationalPublicationFailureKind;
use contracts::mvcc::publication::outcome::refusals::RelationalPublicationDeferred;
use contracts::mvcc::publication::outcome::refusals::RelationalPublicationDenial;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisDescriptor;
use worth_relational_branch::internal_port::branch::reference::RelationalBranchReferenceMutableState;
use worth_relational_branch::internal_port::branch::reference_publication_cell::RelationalBranchPublicationCell;
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use worth_relational_branch::internal_port::history::retention::head_retirement::RelationalHeadRetirementCapacity;
use worth_relational_branch::internal_port::history::retention::lane::RelationalBranchRetentionBinding;

impl PreparedCanonicalBranchMovement {
    #[expect(
        clippy::disallowed_methods,
        reason = "Addresses serve live allocation lookup and custody; canonical order uses declared identity"
    )]
    pub fn select_next_basis(
        &self,
        publication_cell: &RelationalBranchPublicationCell,
        expected: &RelationalBranchBasisDescriptor,
    ) -> Result<
        (
            RelationalBranchReferenceMutableState,
            RelationalBranchBasisDescriptor,
        ),
        RelationalPublicationOutcome,
    > {
        let next_state = self.next_cell.state_snapshot();
        if publication_cell.runtime_instance_id() != expected.runtime_instance_id()
            || publication_cell.branch_id() != expected.branch_id()
        {
            return Err(RelationalPublicationOutcome::denied(
                RelationalPublicationDenial::OwnerMismatch,
            ));
        }
        if self.next_cell.identity().runtime_instance_id() != publication_cell.runtime_instance_id()
            || self.next_cell.identity().branch_id() != publication_cell.branch_id()
            || self.next_cell.root().as_ref().map(Arc::as_ptr) != Some(Arc::as_ptr(&self.root))
        {
            return Err(RelationalPublicationOutcome::failed(
                RelationalPublicationFailure::new(
                    RelationalPublicationFailureKind::PreparedRootMismatch,
                    "prepared next root does not match its branch publication cell",
                ),
            ));
        }
        let next_descriptor = match worth_relational_branch::internal_port::branch::basis_admission::descriptor_for_cell(
            &self.next_cell,
            &self.root,
        ) {
            Ok(descriptor) => descriptor,
            Err(denial) => {
                return Err(RelationalPublicationOutcome::failed(
                    RelationalPublicationFailure::new(
                        RelationalPublicationFailureKind::PreparedBasisDescriptor(denial.clone()),
                        format!("prepared next basis failed before movement: {denial:?}"),
                    ),
                ));
            }
        };
        Ok((next_state, next_descriptor))
    }

    pub fn retain_for_cutover(
        &self,
        next_descriptor: RelationalBranchBasisDescriptor,
        retention_binding: &RelationalBranchRetentionBinding,
        publication_cell: &RelationalBranchPublicationCell,
        expected_root: &Arc<RelationalBranchRoot>,
    ) -> Result<
        (
            AdmittedRelationalBranchBasis,
            RelationalHeadRetirementCapacity,
        ),
        RelationalPublicationOutcome,
    > {
        let next_basis =
            worth_relational_branch::internal_port::branch::basis_admission::issue_admitted_relational_branch_basis_with_retention(
                next_descriptor,
                self.next_cell.identity().clone(),
                Arc::clone(&self.root),
                publication_cell.clone(),
                retention_binding,
            );
        let next_basis = match next_basis.and_then(|basis| publication_cell.register_basis(basis)) {
            Ok(basis) => basis,
            Err(contracts::branch::basis_denial::RelationalBranchBasisDenial::RetentionCapacityExhausted) => {
                return Err(RelationalPublicationOutcome::deferred(
                    RelationalPublicationDeferred::RetentionBackpressure,
                ));
            }
            Err(contracts::branch::basis_denial::RelationalBranchBasisDenial::UnavailableRetainedTarget) => {
                return Err(RelationalPublicationOutcome::denied(
                    RelationalPublicationDenial::OwnerUnavailable {
                        runtime_instance_id: publication_cell.runtime_instance_id(),
                    },
                ));
            }
            Err(denial) => {
                return Err(RelationalPublicationOutcome::failed(
                    RelationalPublicationFailure::new(
                        RelationalPublicationFailureKind::NextBasisAdmission(denial.clone()),
                        format!("next basis admission failed before movement: {denial:?}"),
                    ),
                ));
            }
        };
        let head_retirement = match retention_binding.reserve_head_retirement_capacity(
            self.next_cell.identity(),
            expected_root,
            publication_cell.head_retention(),
        ) {
            Ok(reservation) => reservation,
            Err(
                worth_relational_branch::internal_port::history::retention::owner::RelationalRetentionAcquisitionDenial::CapacityExhausted,
            ) => {
                return Err(RelationalPublicationOutcome::deferred(
                    RelationalPublicationDeferred::RetentionBackpressure,
                ));
            }
            Err(
                worth_relational_branch::internal_port::history::retention::owner::RelationalRetentionAcquisitionDenial::OwnerUnavailable,
            ) => {
                return Err(RelationalPublicationOutcome::denied(
                    RelationalPublicationDenial::OwnerUnavailable {
                        runtime_instance_id: publication_cell.runtime_instance_id(),
                    },
                ));
            }
            Err(
                worth_relational_branch::internal_port::history::retention::owner::RelationalRetentionAcquisitionDenial::IdentityExhausted,
            ) => {
                return Err(RelationalPublicationOutcome::failed(
                    RelationalPublicationFailure::new(
                        RelationalPublicationFailureKind::RetentionIdentityExhausted,
                        "head-retirement retention identity exhausted before movement",
                    ),
                ));
            }
            Err(denial) => {
                return Err(RelationalPublicationOutcome::failed(
                    RelationalPublicationFailure::new(
                        RelationalPublicationFailureKind::RetentionOwner,
                        format!("head-retirement reservation failed before movement: {denial:?}"),
                    ),
                ));
            }
        };
        Ok((next_basis, head_retirement))
    }
}
