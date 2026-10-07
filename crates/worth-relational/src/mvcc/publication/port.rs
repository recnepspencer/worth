use std::sync::Arc;

use super::candidate::PreparedRelationalPublicationParts;
use super::{
    PerformedRelationalCommit, PreparedRelationalCommitCandidate, RelationalPublicationDeferred,
    RelationalPublicationDenial, RelationalPublicationFailure, RelationalPublicationFailureKind,
    RelationalPublicationOutcome, StaleRelationalBranchObservation,
};
use crate::branch::{
    RelationalBranchBasisDescriptor, RelationalBranchPublicationCell,
    RelationalBranchReferenceCell, RelationalBranchRoot,
};

/// Independently borrowable Relational owner publication service.
#[derive(Debug, Clone)]
pub struct RelationalPublicationPort {
    runtime_instance_id: u64,
    configuration: crate::runtime::RelationalRuntimeConfigurationBinding,
    owner_binding: crate::runtime::RelationalRuntimeOwnerBinding,
    publication_binding: crate::runtime::RelationalRuntimePublicationBinding,
    branch_head_versions: crate::runtime::BranchHeadVersionIndexAuthority,
}

pub(crate) struct PreparedCanonicalBranchMovement {
    record_allocations: crate::runtime::PendingRecordAllocations,
    next_cell: RelationalBranchReferenceCell,
    root: Arc<RelationalBranchRoot>,
    canonical_publication_route: crate::runtime::PreparedCanonicalPublicationRoute,
}

struct PreparedBranchPublicationPreflight {
    companion: Option<super::PreparedPublicationCompanionEffect>,
    companion_binding: Option<super::CandidateCompanionBinding>,
    movement: PreparedCanonicalBranchMovement,
    expected: RelationalBranchBasisDescriptor,
    publication_cell: RelationalBranchPublicationCell,
    next_state: crate::branch::RelationalBranchReferenceMutableState,
    next_basis: crate::branch::AdmittedRelationalBranchBasis,
    settlement_reservation: crate::runtime::RelationalPendingSettlementReservation,
    head_retirement: crate::history::retention::RelationalHeadRetirementReservation,
    candidate_retention: crate::history::retention::RelationalCandidateRetentionObligation,
    control: crate::mvcc::RelationalOperationControl,
    expires_at: std::time::Instant,
    maximum_lifetime_millis: u64,
    branch_head_versions: crate::runtime::BranchHeadVersionIndexAuthority,
}

impl RelationalPublicationPort {
    pub(crate) fn new(
        runtime_instance_id: u64,
        configuration: crate::runtime::RelationalRuntimeConfigurationBinding,
        owner_binding: crate::runtime::RelationalRuntimeOwnerBinding,
        publication_binding: crate::runtime::RelationalRuntimePublicationBinding,
        branch_head_versions: crate::runtime::BranchHeadVersionIndexAuthority,
    ) -> Self {
        Self {
            runtime_instance_id,
            configuration,
            owner_binding,
            publication_binding,
            branch_head_versions,
        }
    }

    pub fn compare_and_publish(
        &self,
        candidate: PreparedRelationalCommitCandidate,
    ) -> RelationalPublicationOutcome {
        if candidate.runtime_instance_id() != self.runtime_instance_id
            || !candidate.belongs_to_publication_owner(&self.publication_binding)
        {
            return RelationalPublicationOutcome::denied(
                RelationalPublicationDenial::ForeignRuntime {
                    expected_runtime_instance_id: self.runtime_instance_id,
                    actual_runtime_instance_id: candidate.runtime_instance_id(),
                },
            );
        }
        let Some(_owner_operation) = self.owner_binding.admit() else {
            return RelationalPublicationOutcome::denied(
                RelationalPublicationDenial::OwnerUnavailable {
                    runtime_instance_id: self.runtime_instance_id,
                },
            );
        };
        // Lock order is configuration epoch, then branch publication. Initial
        // installation cannot replace branch roots between this check and CAS.
        let epoch = self.configuration.operation();
        let expected_generation = epoch.schema_contract_runtime.custom_invariant_generation;
        if candidate.custom_invariant_generation() != expected_generation {
            return RelationalPublicationOutcome::denied(
                RelationalPublicationDenial::StaleInvariantGeneration {
                    expected_generation,
                    actual_generation: candidate.custom_invariant_generation(),
                },
            );
        }
        if candidate.lifetime_expired() {
            return RelationalPublicationOutcome::deferred(
                RelationalPublicationDeferred::CandidateLifetimeExpired {
                    maximum_lifetime_millis: candidate.maximum_lifetime_millis,
                },
            );
        }
        // The read epoch spans the entire publication attempt. Registration and
        // removal and true-head cell installation cannot pass a direct writer
        // while its companion is prepared or cutting over. Query preflight
        // takes its branch lookup only after this epoch; cutover installs the
        // prepaid Native payload and invokes no Query lookup or sink.
        let companion_epoch = match self.publication_binding.companion_epoch() {
            Ok(epoch) => epoch,
            Err(_) => {
                return RelationalPublicationOutcome::deferred(
                    RelationalPublicationDeferred::CompanionRegistrationPending,
                )
            }
        };
        match companion_epoch.active() {
            Ok(_) => {}
            Err(super::PublicationCompanionRegistrationStop::RebindRequired) => {
                return RelationalPublicationOutcome::deferred(
                    RelationalPublicationDeferred::CompanionRebindRequired,
                );
            }
            Err(_) => {
                return RelationalPublicationOutcome::deferred(
                    RelationalPublicationDeferred::CompanionRegistrationPending,
                );
            }
        }
        let parts = match candidate.into_publication_parts() {
            Ok(parts) => parts,
            Err(deferred) => return RelationalPublicationOutcome::deferred(deferred),
        };
        let retention_binding = match parts.publication_cell.head_retention().binding() {
            Ok(binding) => binding,
            Err(_) => {
                return RelationalPublicationOutcome::denied(
                    RelationalPublicationDenial::OwnerUnavailable {
                        runtime_instance_id: self.runtime_instance_id,
                    },
                );
            }
        };
        let outcome = PreparedCanonicalBranchMovement::linearize(
            parts,
            retention_binding,
            self.branch_head_versions.clone(),
            &companion_epoch,
        );
        drop(companion_epoch);
        outcome
    }
}

impl PreparedCanonicalBranchMovement {
    pub(crate) fn new(
        record_allocations: crate::runtime::PendingRecordAllocations,
        mut next_cell: RelationalBranchReferenceCell,
        root: Arc<RelationalBranchRoot>,
        canonical_publication_route: crate::runtime::PreparedCanonicalPublicationRoute,
    ) -> Self {
        next_cell.install_root(Arc::clone(&root));
        Self {
            record_allocations,
            next_cell,
            root,
            canonical_publication_route,
        }
    }

    fn linearize(
        parts: PreparedRelationalPublicationParts,
        retention_binding: crate::history::retention::RelationalBranchRetentionBinding,
        branch_head_versions: crate::runtime::BranchHeadVersionIndexAuthority,
        companion_epoch: &super::CompanionRegistrationEpoch<'_>,
    ) -> RelationalPublicationOutcome {
        match Self::preflight(
            parts,
            retention_binding,
            branch_head_versions,
            companion_epoch,
        ) {
            Ok(preflight) => preflight.perform_cutover(),
            Err(outcome) => outcome,
        }
    }
}

impl crate::runtime::RelationalRuntime {
    /// The independently borrowable publication service for this runtime.
    ///
    /// This service owns exactly one operation,
    /// [`RelationalPublicationPort::compare_and_publish`], which is the only
    /// path that moves a branch reference. It consumes the candidate produced
    /// by
    /// [`RelationalRuntime::preparation_port`](crate::runtime::RelationalRuntime::preparation_port)
    /// and returns a typed terminal outcome rather than a `Result`, because a
    /// branch that did not move is not always an error. `Performed` transfers
    /// a settlement obligation to
    /// [`RelationalRuntime::settlement_port`](crate::runtime::RelationalRuntime::settlement_port).
    ///
    /// Different branch cells do not share an ordinary publication lock. The
    /// runnable owner workflow, including the caller-owned response to a
    /// contended patch-position reservation, is
    /// `examples/branch_local_mvcc.rs`.
    pub fn publication_port(&self) -> RelationalPublicationPort {
        RelationalPublicationPort::new(
            self.runtime_instance_id(),
            self.configuration_binding(),
            self.owner_binding(),
            self.publication_binding(),
            self.history.branch_head_version_index(),
        )
    }

    pub fn patch_position_reservation_counters(
        &self,
    ) -> crate::runtime::RelationalPatchPositionReservationCounters {
        self.history.canonical_publication_reservation_counters()
    }
}

#[path = "port_cutover.rs"]
mod cutover;
#[path = "port_preflight.rs"]
mod preflight;
