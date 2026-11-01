//! Shared merge rows retain their own admitted backing through every clone.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::{Arc, OnceLock};
use worth_execution::ExecutionMemoryReservation;

pub(crate) struct MergeWitnessRows<T>(Arc<RowsBacking<T>>);

struct RowsBacking<T> {
    rows: Arc<[T]>,
    // Last: rows and their nested owned data drop before this exact admission.
    memory: OnceLock<Option<ExecutionMemoryReservation>>,
}

impl<T> MergeWitnessRows<T> {
    pub(crate) fn retain_decoded_memory(
        &self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>> {
        self.0.memory.set(memory)
    }

    pub(crate) fn control_bytes() -> Option<u64> {
        let layout = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::new::<RowsBacking<T>>())
            .ok()?
            .0
            .pad_to_align();
        u64::try_from(layout.size()).ok()
    }

    pub(crate) fn storage_bytes(count: usize) -> Option<u64> {
        let rows = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::array::<T>(count).ok()?)
            .ok()?
            .0
            .pad_to_align();
        Self::control_bytes()?.checked_add(u64::try_from(rows.size()).ok()?)
    }
}

impl<T> From<Arc<[T]>> for MergeWitnessRows<T> {
    fn from(rows: Arc<[T]>) -> Self {
        Self(Arc::new(RowsBacking {
            rows,
            memory: OnceLock::new(),
        }))
    }
}
impl<T> Clone for MergeWitnessRows<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T> std::ops::Deref for MergeWitnessRows<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.0.rows
    }
}
impl<T: std::fmt::Debug> std::fmt::Debug for MergeWitnessRows<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.rows.fmt(formatter)
    }
}
impl<T: PartialEq> PartialEq for MergeWitnessRows<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.rows == other.0.rows
    }
}
impl<T: Eq> Eq for MergeWitnessRows<T> {}
impl<T: Serialize> Serialize for MergeWitnessRows<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.rows.serialize(serializer)
    }
}
impl<'de, T: Deserialize<'de>> Deserialize<'de> for MergeWitnessRows<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Arc::<[T]>::deserialize(deserializer).map(Self::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::data::{CommitId, MergeBasisAncestors};
    use crate::identity::data::{EntityId, PartitionId};
    use crate::merge::data::{
        DeclaredIdentityKeys, IdentityBasisName, MergePolicyName, RelationalMergeAdmittedSurfaceRow,
    };
    use crate::transactions::data::RecordRef;
    use worth_foundational::facade::AspectKey;

    #[test]
    fn opaque_merge_leaf_wire_matches_original_shared_storage_in_both_encodings() {
        let identity: Arc<str> = Arc::from("identity-λ");
        let policy: Arc<str> = Arc::from("policy-name");
        let keys: Arc<[AspectKey]> = Arc::from([AspectKey::new("key-λ").unwrap()]);
        let ancestors: Arc<[CommitId]> = Arc::from([CommitId(1), CommitId(u64::MAX)]);
        let rows: Arc<[RelationalMergeAdmittedSurfaceRow]> =
            Arc::from([RelationalMergeAdmittedSurfaceRow::new(
                RecordRef::Entity(EntityId::new(PartitionId(1), 2, 1)),
                None,
            )]);
        let original = (
            identity.clone(),
            policy.clone(),
            keys.clone(),
            ancestors.clone(),
            rows.clone(),
        );
        let opaque = (
            IdentityBasisName::from(identity),
            MergePolicyName::from(policy),
            DeclaredIdentityKeys::from(keys),
            MergeBasisAncestors::from(ancestors),
            MergeWitnessRows::from(rows),
        );
        let old_named = rmp_serde::to_vec_named(&original).unwrap();
        let old_compact = rmp_serde::to_vec(&original).unwrap();
        assert_eq!(rmp_serde::to_vec_named(&opaque).unwrap(), old_named);
        assert_eq!(rmp_serde::to_vec(&opaque).unwrap(), old_compact);
        type Decoded = (
            IdentityBasisName,
            MergePolicyName,
            DeclaredIdentityKeys,
            MergeBasisAncestors,
            MergeWitnessRows<RelationalMergeAdmittedSurfaceRow>,
        );
        for bytes in [old_named, old_compact] {
            let actual: Decoded = rmp_serde::from_slice(&bytes).unwrap();
            assert_eq!(actual, opaque);
        }
    }
}
