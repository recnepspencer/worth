use std::sync::atomic::{AtomicU64, Ordering};

/// One process-local live Runtime World composition owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuntimeWorldOwnerIdentity(u64);

impl RuntimeWorldOwnerIdentity {
    const fn from_ordinal(ordinal: u64) -> Self {
        Self(ordinal)
    }

    /// Descriptive owner ordinal for stable identity derivation.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// The identity family whose checked sequence reached its terminal value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeWorldIdentityFamily {
    Owner,
    ProductBranchReferenceGeneration,
    ProductBranchIncarnation,
    CompositeCommit,
    BootstrapAttempt,
    PublicationAttempt,
    ProductUnpublishedOwnerEffects,
}

/// Identity exhaustion is a typed pre-effect denial. No family wraps or
/// reuses an earlier value after this error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeWorldIdentityExhaustion {
    family: RuntimeWorldIdentityFamily,
}

impl RuntimeWorldIdentityExhaustion {
    pub(crate) const fn new(family: RuntimeWorldIdentityFamily) -> Self {
        Self { family }
    }

    pub const fn family(self) -> RuntimeWorldIdentityFamily {
        self.family
    }
}

static NEXT_OWNER_IDENTITY: AtomicU64 = AtomicU64::new(0);

/// Owner-local checked issuers used by the later Runtime World owner.
///
/// The owner sequence itself is process-unique. All other sequences are
/// intentionally local to one owner, and every emitted value carries that
/// owner identity in its type's value. Composite basis identities are bound
/// from owner-issued component admission identities rather than a cursor.
#[derive(Debug)]
pub(crate) struct RuntimeWorldIdentityIssuer {
    owner: RuntimeWorldOwnerIdentity,
    next_branch_incarnation: u64,
    next_composite_commit: u64,
    next_bootstrap_attempt: u64,
    next_publication_attempt: u64,
    next_product_unpublished: u64,
}

impl RuntimeWorldIdentityIssuer {
    pub(super) fn from_owner_construction(
        capability: &crate::lifecycle::owner::RuntimeWorldOwnerConstructionCapability,
    ) -> Result<(Self, RuntimeWorldOwnerIdentity), RuntimeWorldIdentityExhaustion> {
        let _ = capability;
        let owner = issue_owner_identity()?;
        Ok((
            Self {
                owner,
                next_branch_incarnation: 0,
                next_composite_commit: 0,
                next_bootstrap_attempt: 0,
                next_publication_attempt: 0,
                next_product_unpublished: 0,
            },
            owner,
        ))
    }

    pub(crate) const fn owner(&self) -> RuntimeWorldOwnerIdentity {
        self.owner
    }

    fn next(
        cursor: &mut u64,
        family: RuntimeWorldIdentityFamily,
    ) -> Result<u64, RuntimeWorldIdentityExhaustion> {
        let current = *cursor;
        *cursor = current
            .checked_add(1)
            .ok_or(RuntimeWorldIdentityExhaustion::new(family))?;
        Ok(current)
    }

    pub(crate) fn branch_incarnation(
        &mut self,
    ) -> Result<super::ProductBranchIncarnation, RuntimeWorldIdentityExhaustion> {
        Ok(super::ProductBranchIncarnation::issued(
            self.owner,
            Self::next(
                &mut self.next_branch_incarnation,
                RuntimeWorldIdentityFamily::ProductBranchIncarnation,
            )?,
        ))
    }

    pub(crate) fn composite_basis(
        &self,
        relational: worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity,
        signal: worth_signal::facade::branch::SignalBranchBasisAdmissionIdentity,
        correspondence: worth_runtime_bridge::facade::BridgeCorrespondenceAdmissionIdentity,
    ) -> super::CompositeBasisKey {
        super::CompositeBasisKey::issued(self.owner, relational, signal, correspondence)
    }

    pub(crate) fn composite_commit(
        &mut self,
    ) -> Result<super::CompositeCommitIdentity, RuntimeWorldIdentityExhaustion> {
        Ok(super::CompositeCommitIdentity::issued(
            self.owner,
            Self::next(
                &mut self.next_composite_commit,
                RuntimeWorldIdentityFamily::CompositeCommit,
            )?,
        ))
    }

    pub(crate) fn bootstrap_attempt(
        &mut self,
    ) -> Result<super::RuntimeWorldBootstrapAttemptIdentity, RuntimeWorldIdentityExhaustion> {
        Ok(super::RuntimeWorldBootstrapAttemptIdentity::issued(
            self.owner,
            Self::next(
                &mut self.next_bootstrap_attempt,
                RuntimeWorldIdentityFamily::BootstrapAttempt,
            )?,
        ))
    }

    pub(crate) fn publication_attempt(
        &mut self,
    ) -> Result<super::CompositePublicationAttemptIdentity, RuntimeWorldIdentityExhaustion> {
        Ok(super::CompositePublicationAttemptIdentity::issued(
            self.owner,
            Self::next(
                &mut self.next_publication_attempt,
                RuntimeWorldIdentityFamily::PublicationAttempt,
            )?,
        ))
    }

    pub(crate) fn product_unpublished(
        &mut self,
    ) -> Result<super::ProductUnpublishedOwnerEffectsIdentity, RuntimeWorldIdentityExhaustion> {
        Ok(super::ProductUnpublishedOwnerEffectsIdentity::issued(
            self.owner,
            Self::next(
                &mut self.next_product_unpublished,
                RuntimeWorldIdentityFamily::ProductUnpublishedOwnerEffects,
            )?,
        ))
    }

    #[cfg(test)]
    pub(crate) fn set_next_publication_attempt_for_test(&mut self, next: u64) {
        self.next_publication_attempt = next;
    }
}

fn issue_owner_identity() -> Result<RuntimeWorldOwnerIdentity, RuntimeWorldIdentityExhaustion> {
    let mut current = NEXT_OWNER_IDENTITY.load(Ordering::Relaxed);
    loop {
        let next = current
            .checked_add(1)
            .ok_or(RuntimeWorldIdentityExhaustion::new(
                RuntimeWorldIdentityFamily::Owner,
            ))?;
        match NEXT_OWNER_IDENTITY.compare_exchange_weak(
            current,
            next,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return Ok(RuntimeWorldOwnerIdentity::from_ordinal(current)),
            Err(observed) => current = observed,
        }
    }
}
