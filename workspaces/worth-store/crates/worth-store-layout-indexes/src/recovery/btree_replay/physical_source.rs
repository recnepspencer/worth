use std::sync::atomic::{AtomicU64, Ordering};

use worth_store_contracts::AcceptedHandoffReadiness;
use worth_store_physical_format::{
    InMemoryPhysicalFormatModel, InMemoryPhysicalFormatModelRequest,
    InMemoryPhysicalFormatReplayArtifact, PersistedPhysicalLayout, PhysicalReference,
    PhysicalStoreIdentity,
};
use worth_store_recovery_physics::PhysicalSourceSelection;

use super::{BTreeReplayRootAgreement, BTreeReplaySourceDenial};

static NEXT_BTREE_REPLAY_PHYSICAL_SOURCE_IDENTITY: AtomicU64 = AtomicU64::new(1);

/// Opaque identity issued by the layout owner for one admitted physical replay source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BTreeReplayPhysicalSourceIdentity(u64);

impl BTreeReplayPhysicalSourceIdentity {
    fn issue() -> Self {
        let value = NEXT_BTREE_REPLAY_PHYSICAL_SOURCE_IDENTITY
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .expect("B-tree replay physical source identity space exhausted");
        Self(value)
    }
}

/// Layout-owned capability proving one physical replay source before semantic planning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedBTreeReplayPhysicalSource {
    identity: BTreeReplayPhysicalSourceIdentity,
    readiness: AcceptedHandoffReadiness,
    root_reference: PhysicalReference,
    replay_artifact: InMemoryPhysicalFormatReplayArtifact,
    store_identity: PhysicalStoreIdentity,
    durable_source: PhysicalSourceSelection,
    root_agreement: BTreeReplayRootAgreement,
}

impl AdmittedBTreeReplayPhysicalSource {
    pub fn admit(
        readiness: AcceptedHandoffReadiness,
        root_reference: PhysicalReference,
        replay_artifact: InMemoryPhysicalFormatReplayArtifact,
        expected_store_identity: PhysicalStoreIdentity,
        durable_source: PhysicalSourceSelection,
    ) -> Result<Self, BTreeReplaySourceDenial> {
        let root_agreement = BTreeReplayRootAgreement::admit(&durable_source, root_reference)?;
        replay_artifact.admit_bootstrap_open_witness()?;
        match replay_artifact
            .persisted_layout()
            .root_manifest_candidates()
        {
            [] => return Err(BTreeReplaySourceDenial::RootManifestMissing),
            [_] => {}
            candidates => {
                return Err(BTreeReplaySourceDenial::AmbiguousRootManifest {
                    candidates: candidates.len(),
                });
            }
        }
        let canonical = InMemoryPhysicalFormatModelRequest::physical_format_canonical();
        let request = InMemoryPhysicalFormatModelRequest::for_store(
            canonical.headers().clone(),
            expected_store_identity.clone(),
        );
        let mut facade = replay_artifact.restore_model(readiness.clone(), request)?;
        facade.page_access().locate_record(root_reference)?;
        Ok(Self {
            identity: BTreeReplayPhysicalSourceIdentity::issue(),
            readiness,
            root_reference,
            replay_artifact,
            store_identity: expected_store_identity,
            durable_source,
            root_agreement,
        })
    }

    pub fn bind_intent<I>(self, intent: I) -> AdmittedBTreeReplaySource<I> {
        AdmittedBTreeReplaySource {
            intent,
            physical: self,
        }
    }

    pub const fn identity(&self) -> BTreeReplayPhysicalSourceIdentity {
        self.identity
    }

    pub const fn root_reference(&self) -> PhysicalReference {
        self.root_reference
    }

    pub const fn durable_source(&self) -> &PhysicalSourceSelection {
        &self.durable_source
    }

    pub const fn root_agreement(&self) -> &BTreeReplayRootAgreement {
        &self.root_agreement
    }
}

/// Layout-owned capability binding semantic intent to one admitted physical source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedBTreeReplaySource<I> {
    intent: I,
    physical: AdmittedBTreeReplayPhysicalSource,
}

impl<I> AdmittedBTreeReplaySource<I> {
    pub const fn intent(&self) -> &I {
        &self.intent
    }

    pub const fn root_reference(&self) -> PhysicalReference {
        self.physical.root_reference
    }

    pub const fn persisted_layout(&self) -> &PersistedPhysicalLayout {
        self.physical.replay_artifact.persisted_layout()
    }

    pub const fn durable_source(&self) -> &PhysicalSourceSelection {
        &self.physical.durable_source
    }

    pub const fn root_agreement(&self) -> &BTreeReplayRootAgreement {
        &self.physical.root_agreement
    }

    pub const fn physical_source(&self) -> &AdmittedBTreeReplayPhysicalSource {
        &self.physical
    }

    pub fn reopen(&self) -> Result<InMemoryPhysicalFormatModel, BTreeReplaySourceDenial> {
        let canonical = InMemoryPhysicalFormatModelRequest::physical_format_canonical();
        let request = InMemoryPhysicalFormatModelRequest::for_store(
            canonical.headers().clone(),
            self.physical.store_identity.clone(),
        );
        Ok(self
            .physical
            .replay_artifact
            .restore_model(self.physical.readiness.clone(), request)?)
    }
}

#[cfg(test)]
mod tests {
    use super::BTreeReplayPhysicalSourceIdentity;

    #[test]
    fn independently_issued_physical_sources_never_alias() {
        let first = BTreeReplayPhysicalSourceIdentity::issue();
        let clone_identity = first;
        let second = BTreeReplayPhysicalSourceIdentity::issue();

        assert_eq!(first, clone_identity);
        assert_ne!(first, second);
    }
}
