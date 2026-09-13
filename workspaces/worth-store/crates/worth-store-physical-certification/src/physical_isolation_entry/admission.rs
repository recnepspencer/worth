use worth_store_physical_isolation::{
    PhysicalIsolationEntryDenial, PhysicalIsolationEntryEvidence, PhysicalIsolationEntryIdentity,
    PhysicalIsolationEntryRebindRequired, PhysicalIsolationRootEpochBasis,
};
use worth_store_recovery_physics::PageLsn;
use worth_store_recovery_runtime::RecoveryCompletion;

#[derive(Debug, Clone, Copy)]
pub struct PhysicalIsolationEntryRequest<'a> {
    recovery_completion: &'a RecoveryCompletion,
    store_authority_identity: worth_store_authority::StoreCurrentAuthorityIdentity,
}

impl<'a> PhysicalIsolationEntryRequest<'a> {
    pub fn from_recovery_completion(recovery_completion: &'a RecoveryCompletion) -> Self {
        Self::for_store(
            recovery_completion,
            &worth_store_physical_format::PhysicalStoreIdentity::physical_format_default(),
        )
    }

    pub fn for_store(
        recovery_completion: &'a RecoveryCompletion,
        store_identity: &worth_store_physical_format::PhysicalStoreIdentity,
    ) -> Self {
        Self {
            recovery_completion,
            store_authority_identity: store_identity.authority_identity(),
        }
    }

    pub const fn recovery_completion(&self) -> &'a RecoveryCompletion {
        self.recovery_completion
    }

    pub const fn store_authority_identity(
        &self,
    ) -> worth_store_authority::StoreCurrentAuthorityIdentity {
        self.store_authority_identity
    }
}

#[derive(Debug, Clone)]
pub struct PhysicalIsolationEntryAdmission {
    recovery_completion: RecoveryCompletion,
    identity: PhysicalIsolationEntryIdentity,
    root_epoch_basis: PhysicalIsolationRootEpochBasis,
    evidence: PhysicalIsolationEntryEvidence,
}

pub fn admit_physical_isolation_entry(
    request: PhysicalIsolationEntryRequest<'_>,
) -> Result<PhysicalIsolationEntryAdmission, PhysicalIsolationEntryDenial> {
    PhysicalIsolationEntryAdmission::admit(request)
}

pub fn admit_physical_isolation_entry_checked(
    request: PhysicalIsolationEntryRequest<'_>,
) -> PhysicalIsolationEntryCheckedOutcome {
    match admit_physical_isolation_entry(request) {
        Ok(admission) => PhysicalIsolationEntryCheckedOutcome::Admitted(Box::new(admission)),
        Err(denial) => PhysicalIsolationEntryCheckedOutcome::Denied(denial),
    }
}

#[derive(Debug, Clone)]
pub enum PhysicalIsolationEntryCheckedOutcome {
    Admitted(Box<PhysicalIsolationEntryAdmission>),
    Denied(PhysicalIsolationEntryDenial),
    Stale(PhysicalIsolationEntryDenial),
    RebindRequired(PhysicalIsolationEntryRebindRequired),
}

impl PartialEq for PhysicalIsolationEntryCheckedOutcome {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Admitted(left), Self::Admitted(right)) => left.identity() == right.identity(),
            (Self::Denied(left), Self::Denied(right)) | (Self::Stale(left), Self::Stale(right)) => {
                left == right
            }
            (Self::RebindRequired(left), Self::RebindRequired(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for PhysicalIsolationEntryCheckedOutcome {}

impl PhysicalIsolationEntryAdmission {
    fn admit(
        request: PhysicalIsolationEntryRequest<'_>,
    ) -> Result<Self, PhysicalIsolationEntryDenial> {
        let recovery_completion = request.recovery_completion().clone();
        let identity = PhysicalIsolationEntryIdentity::from_certification_boundary(
            recovery_completion.recovered_root(),
            recovery_completion.admitted_page_lsn_frontier(),
            recovery_completion.source_decision_digest(),
            recovery_completion.replayed_frames(),
            recovery_completion.source_candidate_count(),
            request.store_authority_identity(),
        );
        let root_epoch_basis = identity.root_epoch_basis();
        let evidence = PhysicalIsolationEntryEvidence::from_entry_identity(&identity)?;
        Ok(Self {
            recovery_completion,
            identity,
            root_epoch_basis,
            evidence,
        })
    }

    pub const fn recovery_completion(&self) -> &RecoveryCompletion {
        &self.recovery_completion
    }

    pub const fn identity(&self) -> &PhysicalIsolationEntryIdentity {
        &self.identity
    }

    pub const fn root_epoch_basis(&self) -> PhysicalIsolationRootEpochBasis {
        self.root_epoch_basis
    }

    pub fn recovered_root(&self) -> &str {
        self.recovery_completion.recovered_root()
    }

    pub const fn admitted_page_lsn_frontier(&self) -> Option<PageLsn> {
        self.recovery_completion.admitted_page_lsn_frontier()
    }

    pub const fn replayed_frames(&self) -> usize {
        self.recovery_completion.replayed_frames()
    }

    pub const fn source_candidate_count(&self) -> usize {
        self.recovery_completion.source_candidate_count()
    }

    pub const fn evidence(&self) -> &PhysicalIsolationEntryEvidence {
        &self.evidence
    }

    pub const fn is_store_physical_stability_authority(&self) -> bool {
        false
    }
}
