//! Exact physical effect keys. Empty coordinate lists are not footprints.

mod admission;
mod intersection;
mod lowering;
#[cfg(test)]
mod tests;

pub(in crate::physical_runtime) use admission::{
    PhysicalEffectAdmission, PhysicalEffectAdmissionDenial, PhysicalEffectAdmissionLease,
};
pub(in crate::physical_runtime) use intersection::{effect_relation, shares_coordination_identity};
pub(in crate::physical_runtime) use lowering::lower_effect_footprint;

use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::{PhysicalWorkObligationIdentity, RecordArtifactFile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalEffectAccess {
    Read,
    Write,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalEffectKey {
    Range {
        artifact: RecordArtifactFile,
        start: u64,
        end: u64,
        access: PhysicalEffectAccess,
    },
    WholeArtifact {
        artifact: RecordArtifactFile,
        access: PhysicalEffectAccess,
    },
    DeleteArtifact {
        artifact: RecordArtifactFile,
    },
    Wal {
        segment: u64,
        generation: u64,
        start: u64,
        end: u64,
        access: PhysicalEffectAccess,
    },
    DeleteWal {
        segment: u64,
        generation: u64,
    },
    Allocator {
        generation: u64,
        block: Option<u64>,
    },
    RootPublication,
    Namespace,
    Checkpoint {
        start: u64,
        end: u64,
        whole: bool,
        access: PhysicalEffectAccess,
    },
    Obligation {
        identity: PhysicalWorkObligationIdentity,
        access: PhysicalEffectAccess,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime) struct PhysicalEffectFootprint {
    store: StableStoreIdentity,
    keys: Box<[PhysicalEffectKey]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalEffectRelation {
    Disjoint,
    SharedRead,
    Conflict,
}

impl PhysicalEffectFootprint {
    pub(in crate::physical_runtime) fn new(
        store: StableStoreIdentity,
        keys: Vec<PhysicalEffectKey>,
    ) -> Self {
        assert!(
            !keys.is_empty(),
            "an admitted physical effect names at least one coordination key"
        );
        Self {
            store,
            keys: keys.into_boxed_slice(),
        }
    }

    pub(in crate::physical_runtime) const fn store(&self) -> StableStoreIdentity {
        self.store
    }

    pub(in crate::physical_runtime) fn keys(&self) -> &[PhysicalEffectKey] {
        &self.keys
    }
}
