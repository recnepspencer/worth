use std::sync::atomic::{AtomicU64, Ordering};

/// A process-local live owner handle. It never implements DurableIdentity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EphemeralWorldOwnerIdentity(u64);
impl EphemeralWorldOwnerIdentity {
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum OwnerScope {
    Ephemeral(EphemeralWorldOwnerIdentity),
    Home(worth_foundational::facade::DurableWorldOwnerIdentity),
}

/// Live identity carriage. Home authority equality includes its admitted
/// incarnation. Durable data uses the separate Foundation identity instead.
#[derive(Debug, Clone, Copy)]
pub struct RuntimeWorldOwnerIdentity {
    scope: OwnerScope,
    instance: EphemeralWorldOwnerIdentity,
    home_fence: Option<worth_proof::HomeIncarnationFence>,
}
impl RuntimeWorldOwnerIdentity {
    const fn from_ordinal(ordinal: u64) -> Self {
        let instance = EphemeralWorldOwnerIdentity(ordinal);
        Self {
            scope: OwnerScope::Ephemeral(instance),
            instance,
            home_fence: None,
        }
    }
    /// Process-local observation ordinal, not a durable identity encoding.
    pub const fn get(self) -> u64 {
        self.instance.get()
    }
    pub const fn durable(self) -> Option<worth_foundational::facade::DurableWorldOwnerIdentity> {
        match self.scope {
            OwnerScope::Ephemeral(_) => None,
            OwnerScope::Home(home) => Some(home),
        }
    }
    pub(crate) fn for_admitted_home(
        home: worth_foundational::facade::DurableWorldOwnerIdentity,
        incarnation: worth_foundational::facade::HomeIncarnation,
    ) -> Result<Self, RuntimeWorldIdentityExhaustion> {
        let instance = issue_owner_identity()?.instance;
        Ok(Self {
            scope: OwnerScope::Home(home),
            instance,
            home_fence: Some(incarnation.fence()),
        })
    }

    /// The fence carried by live Home authorities; never a durable identity.
    pub const fn home_incarnation_fence(self) -> Option<worth_proof::HomeIncarnationFence> {
        self.home_fence
    }

    pub fn admit_incarnation(
        self,
        presented: Self,
    ) -> Result<(), worth_proof::HomeIncarnationDenial> {
        if let (OwnerScope::Home(current), OwnerScope::Home(other)) = (self.scope, presented.scope)
            && current != other
        {
            return Err(worth_proof::HomeIncarnationDenial::ForeignHome);
        }
        match (self.home_fence, presented.home_fence) {
            (Some(current), Some(presented)) => current.admit(presented),
            (None, None) if self.scope == presented.scope => Ok(()),
            (None, None) | (Some(_), None) | (None, Some(_)) => {
                Err(worth_proof::HomeIncarnationDenial::ForeignHome)
            }
        }
    }

    pub(crate) fn admit_branch_authority(
        self,
        presented: Self,
    ) -> Result<(), crate::branch::RuntimeWorldBranchAdmissionDenial> {
        use crate::branch::RuntimeWorldBranchAdmissionDenial as Denial;
        self.admit_incarnation(presented)
            .map_err(|cause| match cause {
                worth_proof::HomeIncarnationDenial::ForeignHome => Denial::ForeignOwner,
                worth_proof::HomeIncarnationDenial::StaleIncarnation => Denial::StaleIncarnation,
            })?;
        if self != presented {
            return Err(Denial::ForeignOwner);
        }
        Ok(())
    }
}
impl PartialEq for RuntimeWorldOwnerIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.scope == other.scope && self.home_fence == other.home_fence
    }
}
impl Eq for RuntimeWorldOwnerIdentity {}
impl PartialOrd for RuntimeWorldOwnerIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for RuntimeWorldOwnerIdentity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.scope, self.home_fence).cmp(&(other.scope, other.home_fence))
    }
}
impl std::hash::Hash for RuntimeWorldOwnerIdentity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::hash::Hash::hash(&self.scope, state);
        std::hash::Hash::hash(&self.home_fence, state);
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
    pub(super) owner: RuntimeWorldOwnerIdentity,
    pub(super) next_branch_incarnation: u64,
    pub(super) next_composite_commit: u64,
    pub(super) next_bootstrap_attempt: u64,
    pub(super) next_publication_attempt: u64,
    pub(super) next_product_unpublished: u64,
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
