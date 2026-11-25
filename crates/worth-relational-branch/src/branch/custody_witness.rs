//! Read-only custody witness of one admitted basis and its immutable root.
//!
//! A witness holds only weak identities and a copied schema digest. It never
//! extends custody, never yields a basis or root, and grants no currentness or
//! reclamation authority.
//! Custody courts use it to observe whether the actual owners (leases, capture
//! holds, acknowledgement progress) still retain that exact observation.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::basis::WeakAdmittedRelationalBranchBasisInner;
use crate::branch::root::schema::RelationalBranchRootSchemaAuthority;
use crate::branch::root::RelationalBranchRoot;
use crate::visibility::snapshot_observation::SnapshotObservationInputs;
use std::sync::{Arc, Weak};

/// Weak identity of one admitted basis observation and its root.
#[derive(Debug, Clone)]
pub struct RelationalBasisCustodyWitness {
    basis: WeakAdmittedRelationalBranchBasisInner,
    root: Weak<RelationalBranchRoot>,
    schema_authority: Weak<RelationalBranchRootSchemaAuthority>,
    schema_authority_digest: [u8; 32],
}

impl AdmittedRelationalBranchBasis {
    /// Witness this exact observation without retaining it.
    pub fn custody_witness(&self) -> RelationalBasisCustodyWitness {
        RelationalBasisCustodyWitness {
            basis: self.inner.downgrade(),
            root: Arc::downgrade(&self.inner.root),
            schema_authority: Arc::downgrade(&self.inner.root.retained_schema_authority()),
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

    /// Whether both witnesses name the same immutable root allocation.
    pub fn same_root(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.root, &other.root)
    }

    /// Whether both witnesses name the same installed schema authority allocation.
    pub fn same_schema_authority(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.schema_authority, &other.schema_authority)
    }

    /// The witnessed root's installed schema authority digest.
    pub fn schema_authority_digest(&self) -> [u8; 32] {
        self.schema_authority_digest
    }

    /// On `runtime`, the witnessed observation's owner, demand every region of
    /// the still-retained observation fully, then take and release one read
    /// snapshot of it. A foreign runtime refuses before any effect. Returns the retained root's schema
    /// authority digest, or `None` when no owner retains the observation. The
    /// transient strong reference ends before this returns; residency loads
    /// are charged to `execution` like any other demand.
    pub fn read_retained_full<'a>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        owner: impl Into<SnapshotObservationInputs<'a>>,
    ) -> Result<Option<[u8; 32]>, RelationalBasisCustodyReadDenial> {
        let Some(inner) = self.basis.upgrade() else {
            return Ok(None);
        };
        let owner = owner.into();
        let held = AdmittedRelationalBranchBasis { inner };
        // Ownership precedes every effect: a foreign runtime's symbols and
        // residency are never touched for another runtime's observation.
        if held.observation().identity().runtime_instance_id() != owner.runtime_identity {
            return Err(RelationalBasisCustodyReadDenial::ForeignRuntime);
        }
        held.inner
            .root
            .demand_all_full(execution, owner.symbols)
            .map_err(|failure| RelationalBasisCustodyReadDenial::Demand(failure.into()))?;
        let read = owner
            .snapshot_for_observation(&held.observation())
            .map_err(|_| RelationalBasisCustodyReadDenial::Snapshot)?;
        owner
            .release_snapshot(&read)
            .map_err(|_| RelationalBasisCustodyReadDenial::Snapshot)?;
        Ok(Some(held.inner.root.schema_authority().authority_digest()))
    }
}

/// A witnessed read refused; the witness and its owners are unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBasisCustodyReadDenial {
    ForeignRuntime,
    Demand(crate::execution::denial_cause::RelationalExecutionDenialCause),
    Snapshot,
}
