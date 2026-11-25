//! Canonical descriptive membership writes and their separate live issuance.
use crate::history::access::relevant_writes::RelationalAuthoredEntityIndexDefinition;
use crate::identity::data::{EntityId, KindId};
use crate::internal_port::history::access::relevant_writes::selection::RelationalAuthoredEntityIndexDefinitionOwnerPort as _;
use crate::storage::data::AuthoritativeFieldComparisonKey;
use serde::{Deserialize, Serialize};
use std::{alloc::Layout, sync::Arc};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalAuthoredIndexWrite {
    pub definition: RelationalAuthoredEntityIndexDefinition,
    pub entity: EntityId,
    pub old_kind: Option<KindId>,
    pub new_kind: Option<KindId>,
    pub old_key: Option<AuthoritativeFieldComparisonKey>,
    pub new_key: Option<AuthoritativeFieldComparisonKey>,
}

impl CanonicalAuthoredIndexWrite {
    pub fn owned_bytes(&self) -> u64 {
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
    coverage: crate::runtime::state::subsystems::indexing::catalog_coverage::IndexCatalogCoverage,
}

#[derive(Debug)]
struct Backing {
    entries: Vec<CanonicalAuthoredIndexWrite>,
    /// The catalog prefix (`next_index_id`) the entries were prepared against.
    /// `None` is a legacy packet without provenance; it is never certified.
    persisted_bound: Option<u64>,
    verified: Option<LiveIndexWriteVerification>,
    // Last: all original-funded canonical backing dies before its payer.
    memory: Option<worth_execution::ExecutionMemoryReservation>,
}

/// Missing old evidence and parsed data never establish completeness by
/// themselves; only the catalog owner certifies a persisted bound.
#[derive(Clone, Debug, Default)]
pub struct CanonicalAuthoredIndexWrites(Option<Arc<Backing>>);

impl PartialEq for CanonicalAuthoredIndexWrites {
    fn eq(&self, other: &Self) -> bool {
        self.persisted_entries() == other.persisted_entries()
    }
}
impl Eq for CanonicalAuthoredIndexWrites {}

#[derive(Serialize)]
enum WireRef<'a> {
    Unavailable,
    Complete(&'a [CanonicalAuthoredIndexWrite]),
    Covered {
        covered_next_index_id: u64,
        entries: &'a [CanonicalAuthoredIndexWrite],
    },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
enum Wire {
    Unavailable,
    Complete(Vec<CanonicalAuthoredIndexWrite>),
    Covered {
        covered_next_index_id: u64,
        entries: Vec<CanonicalAuthoredIndexWrite>,
    },
}
impl Serialize for CanonicalAuthoredIndexWrites {
    /// Dispatches on the persisted bound only, so a decoded packet re-encodes exactly.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match &self.0 {
            None => WireRef::Unavailable,
            Some(data) => match data.persisted_bound {
                None => WireRef::Complete(&data.entries),
                Some(covered_next_index_id) => WireRef::Covered {
                    covered_next_index_id,
                    entries: &data.entries,
                },
            },
        }
        .serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for CanonicalAuthoredIndexWrites {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (entries, persisted_bound) = match Wire::deserialize(deserializer)? {
            Wire::Unavailable => return Ok(Self::default()),
            Wire::Complete(entries) => (entries, None),
            Wire::Covered {
                covered_next_index_id,
                entries,
            } => (entries, Some(covered_next_index_id)),
        };
        Ok(Self(Some(Arc::new(Backing {
            entries,
            persisted_bound,
            verified: None,
            memory: None,
        }))))
    }
}

impl CanonicalAuthoredIndexWrites {
    /// Attach only before the freshly deserialized private backing can alias.
    /// Alias/slot refusal returns the exact original unconsumed reservation.
    pub fn retain_decoded_memory(
        &mut self,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Result<(), Option<worth_execution::ExecutionMemoryReservation>> {
        let Some(backing) = self.0.as_mut() else {
            return if memory.is_none() {
                Ok(())
            } else {
                Err(memory)
            };
        };
        let Some(backing) = Arc::get_mut(backing) else {
            return Err(memory);
        };
        if backing.memory.is_some() {
            return Err(memory);
        }
        backing.memory = memory;
        Ok(())
    }
    /// The canonical omitted tail means unavailable, never complete-empty.
    pub(super) fn unavailable() -> Self {
        Self(None)
    }

    pub fn is_unavailable(&self) -> bool {
        self.0.is_none()
    }

    /// Only the actual selected/working preparation owner invokes this issuer.
    /// `covered_next_index_id` is the captured catalog prefix the entries cover.
    pub fn prepared(
        entries: Vec<CanonicalAuthoredIndexWrite>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
        coverage: crate::runtime::state::subsystems::indexing::catalog_coverage::IndexCatalogCoverage,
        covered_next_index_id: u64,
    ) -> Self {
        Self(Some(Arc::new(Backing {
            entries,
            persisted_bound: Some(covered_next_index_id),
            verified: Some(LiveIndexWriteVerification { coverage }),
            memory,
        })))
    }

    /// A legacy packet (no persisted bound) carries no provenance.
    pub fn is_legacy(&self) -> bool {
        self.0
            .as_ref()
            .is_some_and(|data| data.persisted_bound.is_none())
    }

    /// A persisted bound must name the same catalog prefix as the commit's
    /// definition delta; anything else is a corrupt envelope.
    pub(super) fn bound_agrees_with(&self, delta_next_index_id: u64) -> bool {
        self.0
            .as_ref()
            .and_then(|data| data.persisted_bound)
            .is_none_or(|bound| bound == delta_next_index_id)
    }

    /// The persisted bound and entries, whether or not live-verified. Only the
    /// catalog owner turns a persisted bound into coverage.
    pub fn persisted_entries(&self) -> Option<(Option<u64>, &[CanonicalAuthoredIndexWrite])> {
        let data = self.0.as_ref()?;
        Some((data.persisted_bound, &data.entries))
    }

    pub fn verified_entries(&self) -> Option<&[CanonicalAuthoredIndexWrite]> {
        let data = self.0.as_ref()?;
        data.verified.as_ref()?;
        Some(&data.entries)
    }

    pub fn coverage(
        &self,
    ) -> Option<crate::runtime::state::subsystems::indexing::catalog_coverage::IndexCatalogCoverage>
    {
        Some(self.0.as_ref()?.verified.as_ref()?.coverage)
    }

    pub fn root_bytes() -> Option<u64> {
        Some(
            Layout::new::<(usize, usize)>()
                .extend(Layout::new::<Backing>())
                .ok()?
                .0
                .pad_to_align()
                .size() as u64,
        )
    }

    pub fn unpaid_owned_bytes(&self) -> u64 {
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
