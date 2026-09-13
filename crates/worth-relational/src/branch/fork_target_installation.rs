use std::sync::Arc;

use super::{PreparedForkTarget, RelationalForkDenial, RelationalForkOutcome, ValidatedForkSource};

impl super::RelationalForkPort {
    pub(super) fn install_fork_target(
        &self,
        source: ValidatedForkSource,
        mut target: PreparedForkTarget,
        return_basis: bool,
    ) -> Result<
        (
            RelationalForkOutcome,
            Option<super::super::AdmittedRelationalBranchBasis>,
        ),
        RelationalForkDenial,
    > {
        let fork_materialized_authoritative_bytes = target
            .target_cell
            .root()
            .filter(|target_root| !Arc::ptr_eq(target_root, &target.source_root))
            .map(|target_root| target_root.logical_partition_payload_bytes())
            .unwrap_or(0);
        let copied_commit_envelopes = target
            .target_cell
            .root()
            .filter(|target_root| !target_root.shares_canonical_envelope_with(&target.source_root))
            .map_or(0, |_| 1);
        let materialization_cost = crate::runtime::RelationalForkMaterializationCost {
            entity_count: 0,
            relation_count: 0,
            authoritative_bytes: fork_materialized_authoritative_bytes,
            copied_commit_envelopes,
        };
        let target_root = target
            .target_cell
            .root()
            .expect("prepared fork target retains its selected root");
        self.owner
            .bind_target_basis_registry_metrics(&mut target.target_cell);
        self.owner
            .install_head(&target.target_cell, &target_root)
            .map_err(map_retention_denial)?;
        let basis = if return_basis {
            let descriptor = crate::branch::descriptor_for_cell(&target.target_cell, &target_root)
                .map_err(map_basis_denial)?;
            let retention_binding = target
                .target_cell
                .head_retention()
                .binding()
                .map_err(map_retention_denial)?;
            let basis = crate::branch::issue_admitted_relational_branch_basis_with_retention(
                descriptor,
                target.target_cell.identity().clone(),
                Arc::clone(&target_root),
                target.target_cell.publication_cell(),
                &retention_binding,
            )
            .map_err(map_basis_denial)?;
            Some(
                target
                    .target_cell
                    .register_basis(basis)
                    .map_err(map_basis_denial)?,
            )
        } else {
            None
        };
        self.owner.record_install(
            &target.target_cell,
            materialization_cost,
            target.source_head_version,
        );
        let target_identity = target.target_cell.identity().clone();
        self.owner
            .install_target(target.reservation, target.target_cell);
        Ok((
            RelationalForkOutcome {
                target_identity,
                source_observation: source.source_observation,
                target_observation: target.target_observation,
                fork_provenance: target.fork_provenance,
                source_truth_version: source.source_truth_version,
                target_truth_version: target.target_truth_version,
                shared_commit_id: target.shared_commit_id,
            },
            basis,
        ))
    }
}

fn map_retention_denial(
    denial: crate::history::retention::RelationalRetentionAcquisitionDenial,
) -> RelationalForkDenial {
    match denial {
        crate::history::retention::RelationalRetentionAcquisitionDenial::CapacityExhausted => {
            RelationalForkDenial::RetentionCapacityExhausted
        }
        crate::history::retention::RelationalRetentionAcquisitionDenial::OwnerUnavailable => {
            RelationalForkDenial::RetentionOwnerUnavailable
        }
        crate::history::retention::RelationalRetentionAcquisitionDenial::IdentityExhausted => {
            RelationalForkDenial::RetentionIdentityExhausted
        }
        crate::history::retention::RelationalRetentionAcquisitionDenial::RootSetTooLarge => {
            RelationalForkDenial::RetentionInvariantViolation
        }
    }
}

fn map_basis_denial(denial: crate::branch::RelationalBranchBasisDenial) -> RelationalForkDenial {
    match denial {
        crate::branch::RelationalBranchBasisDenial::RetentionCapacityExhausted => {
            RelationalForkDenial::RetentionCapacityExhausted
        }
        crate::branch::RelationalBranchBasisDenial::RetentionIdentityExhausted => {
            RelationalForkDenial::RetentionIdentityExhausted
        }
        crate::branch::RelationalBranchBasisDenial::UnavailableRetainedTarget
        | crate::branch::RelationalBranchBasisDenial::OwnerUnavailable => {
            RelationalForkDenial::RetentionOwnerUnavailable
        }
        _ => RelationalForkDenial::RetentionInvariantViolation,
    }
}
