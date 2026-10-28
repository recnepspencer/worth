//! Canonical descriptive membership writes and their separate live issuance.
use crate::history::RelationalAuthoredEntityIndexDefinition;
use crate::identity::data::{EntityId, KindId};
use crate::storage::data::AuthoritativeFieldComparisonKey;
use serde::{Deserialize, Serialize};
use std::{alloc::Layout, sync::Arc};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CanonicalAuthoredIndexWrite {
    pub(crate) definition: RelationalAuthoredEntityIndexDefinition,
    pub(crate) entity: EntityId,
    pub(crate) old_kind: Option<KindId>,
    pub(crate) new_kind: Option<KindId>,
    pub(crate) old_key: Option<AuthoritativeFieldComparisonKey>,
    pub(crate) new_key: Option<AuthoritativeFieldComparisonKey>,
}

impl CanonicalAuthoredIndexWrite {
    pub(crate) fn owned_bytes(&self) -> u64 {
        self.definition
            .owned_bytes()
            .saturating_add(self.old_key.as_ref().map_or(
                0,
                AuthoritativeFieldComparisonKey::owned_allocation_capacity_bytes,
            ))
            .saturating_add(self.new_key.as_ref().map_or(
                0,
                AuthoritativeFieldComparisonKey::owned_allocation_capacity_bytes,
            ))
    }
}

#[derive(Debug)]
struct LiveIndexWriteVerification {
    coverage: crate::runtime::IndexCatalogCoverage,
}

#[derive(Debug)]
struct Backing {
    entries: Vec<CanonicalAuthoredIndexWrite>,
    verified: Option<LiveIndexWriteVerification>,
    // Last: all original-funded canonical backing dies before its payer.
    memory: Option<worth_execution::ExecutionMemoryReservation>,
}

/// Missing old evidence and parsed complete data never establish completeness.
#[derive(Clone, Debug, Default)]
pub(crate) struct CanonicalAuthoredIndexWrites(Option<Arc<Backing>>);

impl PartialEq for CanonicalAuthoredIndexWrites {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref().map(|data| &data.entries) == other.0.as_ref().map(|data| &data.entries)
    }
}
impl Eq for CanonicalAuthoredIndexWrites {}

#[derive(Serialize)]
enum WireRef<'a> {
    Unavailable,
    Complete(&'a [CanonicalAuthoredIndexWrite]),
}
#[derive(Deserialize)]
enum Wire {
    Unavailable,
    Complete(Vec<CanonicalAuthoredIndexWrite>),
}
impl Serialize for CanonicalAuthoredIndexWrites {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match &self.0 {
            None => WireRef::Unavailable,
            Some(data) => WireRef::Complete(&data.entries),
        }
        .serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for CanonicalAuthoredIndexWrites {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match Wire::deserialize(deserializer)? {
            Wire::Unavailable => Self::default(),
            Wire::Complete(entries) => Self(Some(Arc::new(Backing {
                entries,
                verified: None,
                memory: None,
            }))),
        })
    }
}

impl CanonicalAuthoredIndexWrites {
    pub(crate) fn is_unavailable(&self) -> bool {
        self.0.is_none()
    }

    /// Only the actual selected/working preparation owner invokes this issuer.
    pub(crate) fn prepared(
        entries: Vec<CanonicalAuthoredIndexWrite>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
        coverage: crate::runtime::IndexCatalogCoverage,
    ) -> Self {
        Self(Some(Arc::new(Backing {
            entries,
            verified: Some(LiveIndexWriteVerification { coverage }),
            memory,
        })))
    }

    pub(crate) fn verified_entries(&self) -> Option<&[CanonicalAuthoredIndexWrite]> {
        let data = self.0.as_ref()?;
        data.verified.as_ref()?;
        Some(&data.entries)
    }

    pub(crate) fn coverage(&self) -> Option<crate::runtime::IndexCatalogCoverage> {
        Some(self.0.as_ref()?.verified.as_ref()?.coverage)
    }

    pub(crate) fn root_bytes() -> Option<u64> {
        Some(
            Layout::new::<(usize, usize)>()
                .extend(Layout::new::<Backing>())
                .ok()?
                .0
                .pad_to_align()
                .size() as u64,
        )
    }

    pub(crate) fn unpaid_owned_bytes(&self) -> u64 {
        self.0.as_ref().map_or(0, |data| {
            if data.memory.is_some() {
                return 0;
            }
            Self::root_bytes()
                .unwrap_or(u64::MAX)
                .saturating_add(
                    (data.entries.capacity() as u64)
                        .saturating_mul(std::mem::size_of::<CanonicalAuthoredIndexWrite>() as u64),
                )
                .saturating_add(
                    data.entries
                        .iter()
                        .map(CanonicalAuthoredIndexWrite::owned_bytes)
                        .fold(0u64, u64::saturating_add),
                )
        })
    }
}
