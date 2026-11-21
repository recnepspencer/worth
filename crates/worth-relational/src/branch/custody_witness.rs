//! Read-only custody witness of one admitted basis and its immutable root.
//!
//! A witness holds only weak identities. It never extends custody, never
//! yields a basis or root, and grants no currentness or reclamation authority.
//! Custody courts use it to observe whether the actual owners (leases, capture
//! holds, acknowledgement progress) still retain that exact observation.
use super::{AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisInner};
use crate::branch::RelationalBranchRoot;
use std::sync::{Arc, Weak};

/// Weak identity of one admitted basis observation and its root.
#[derive(Debug, Clone)]
pub struct RelationalBasisCustodyWitness {
    basis: Weak<AdmittedRelationalBranchBasisInner>,
    root: Weak<RelationalBranchRoot>,
    schema_authority_digest: [u8; 32],
}

impl AdmittedRelationalBranchBasis {
    /// Witness this exact observation without retaining it.
    pub fn custody_witness(&self) -> RelationalBasisCustodyWitness {
        RelationalBasisCustodyWitness {
            basis: Arc::downgrade(&self.inner),
            root: Arc::downgrade(&self.inner.root),
            schema_authority_digest: self.inner.root.schema_authority().authority_digest(),
        }
    }
}

impl RelationalBasisCustodyWitness {
    /// Whether some owner still retains the witnessed basis observation.
    pub fn basis_retained(&self) -> bool {
        self.basis.strong_count() != 0
    }

    /// Whether some owner still retains the witnessed immutable root.
    pub fn root_retained(&self) -> bool {
        self.root.strong_count() != 0
    }

    /// The witnessed root's installed schema authority digest.
    pub fn schema_authority_digest(&self) -> [u8; 32] {
        self.schema_authority_digest
    }

    /// Demand every region of the still-retained observation fully, then take
    /// and release one read snapshot of it. Returns the retained root's schema
    /// authority digest, or `None` when no owner retains the observation. The
    /// transient strong reference ends before this returns; residency loads
    /// are charged to `execution` like any other demand.
    pub fn read_retained_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Result<Option<[u8; 32]>, RelationalBasisCustodyReadDenial> {
        let Some(inner) = self.basis.upgrade() else {
            return Ok(None);
        };
        let held = AdmittedRelationalBranchBasis { inner };
        held.inner
            .root
            .demand_all_full(execution, &runtime.services.symbols)
            .map_err(|failure| RelationalBasisCustodyReadDenial::Demand(failure.into()))?;
        let read = runtime
            .snapshots()
            .snapshot_for_observation(&held.observation())
            .map_err(|_| RelationalBasisCustodyReadDenial::Snapshot)?;
        runtime
            .snapshots()
            .release_snapshot(&read)
            .map_err(|_| RelationalBasisCustodyReadDenial::Snapshot)?;
        Ok(Some(held.inner.root.schema_authority().authority_digest()))
    }
}

/// A witnessed read refused; the witness and its owners are unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBasisCustodyReadDenial {
    Demand(crate::execution::RelationalExecutionDenialCause),
    Snapshot,
}
