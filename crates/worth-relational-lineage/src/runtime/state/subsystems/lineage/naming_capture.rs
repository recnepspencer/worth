//! Capture of the runtime's naming state as `CHK24` naming pages.
//!
//! Freezing takes `O(branches + schemes)` copy-on-write handles under the
//! lineage read lock, preserving each component's exact actual cold base
//! (`None` for pure-resident components, `Some(link)` only when actually
//! backed by a cold checkpoint root). Sealing merges each branch's resident
//! changes over its common cold base (`O(touched)` pages) or materializes
//! per-component rows against the pinned store when a branch mixes resident
//! and cold components (e.g. a cold historical fork with shared cold groups).

use std::collections::{BTreeMap, BTreeSet};

use worth_relational_storage::internal_port::storage::residency::chunked::member_io::ChunkMemberSink;

use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use crate::lineage::groups::{BranchGroupState, GroupStateDenial};
use crate::lineage::naming_pages::builder::seal_pages;
use crate::lineage::naming_pages::tables::{branch_key_digest, row_key, GROUP_LOG};
use crate::lineage::naming_pages::{
    BranchNamingRoot, CheckpointNamingState, GroupStateHeader, NameIndexHeader, NamingStateHeader,
    SchemeHeader,
};
use crate::lineage::tiered::{NamingBaseLink, PinnedNamingStore};

use super::group_states::PublishedGroupLineage;
use super::name_indexes::BranchNameIndex;
use super::naming_rebuild::BranchNaming;
use super::LineageSubsystem;

struct FrozenBranch {
    naming: Option<BranchNaming>,
    index: Option<BranchNameIndex>,
    groups: Option<Result<BranchGroupState, GroupStateDenial>>,
}

impl FrozenBranch {
    /// Returns `Some(common)` if every present live component shares the exact
    /// same cold base (`None` or `Some(same_link)`), or `None` when components
    /// have mixed cold bases (e.g. truncated resident naming + shared cold groups).
    fn unified_base(&self) -> Option<Option<&NamingBaseLink>> {
        let mut bases: Vec<Option<&NamingBaseLink>> = Vec::with_capacity(3);
        if let Some(Ok(naming)) = &self.naming {
            bases.push(naming.base());
        }
        if let Some(BranchNameIndex::Current(index)) = &self.index {
            bases.push(index.base());
        }
        if let Some(Ok(groups)) = &self.groups {
            bases.push(groups.base());
        }
        let first = bases.first().copied().unwrap_or(None);
        if bases.iter().all(|b| *b == first) {
            Some(first)
        } else {
            None
        }
    }
}

/// The runtime's naming state frozen for one capture.
///
/// Pins the exact checkpoint naming store (and its directory custody) active
/// at freeze time so a concurrent checkpoint ACK cannot swap the base out
/// from underneath this capture before sealing finishes.
pub struct FrozenLineageNaming {
    branches: BTreeMap<BranchId, FrozenBranch>,
    branch_generations: BTreeMap<BranchId, u64>,
    group_log: Vec<PublishedGroupLineage>,
    cold: Option<PinnedNamingStore>,
}

impl std::fmt::Debug for FrozenLineageNaming {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FrozenLineageNaming")
            .field("branches", &self.branches.len())
            .field("group_log", &self.group_log.len())
            .finish_non_exhaustive()
    }
}

impl LineageSubsystem {
    /// Freezes every branch's naming state for capture, preserving each
    /// component's exact actual cold base without falling back to global state.
    ///
    /// Complexity: O(branches + schemes + resident group records).
    pub fn freeze_naming(&self) -> FrozenLineageNaming {
        let state = self.state.read();
        let mut ids: BTreeSet<BranchId> = state.branch_naming_states.keys().cloned().collect();
        ids.extend(state.name_indexes.by_branch.keys().cloned());
        ids.extend(state.group_states.by_branch.keys().cloned());
        let branches: BTreeMap<BranchId, FrozenBranch> =
            ids.into_iter()
                .map(|branch| {
                    let frozen =
                        FrozenBranch {
                            naming: state.branch_naming_states.get(&branch).map(|slot| {
                                slot.as_ref()
                                    .map(|naming| naming.fork(naming.base().cloned()))
                                    .map_err(|denial| *denial)
                            }),
                            index: state.name_indexes.by_branch.get(&branch).map(
                                |index| match index {
                                    BranchNameIndex::Current(index) => BranchNameIndex::Current(
                                        index.fork_to(branch.clone(), index.base().cloned()),
                                    ),
                                    BranchNameIndex::Unavailable(denial) => {
                                        BranchNameIndex::Unavailable(denial.clone())
                                    }
                                },
                            ),
                            groups: state.group_states.by_branch.get(&branch).map(|groups| {
                                match groups {
                                    Ok(groups) => Ok(groups.fork(groups.base().cloned())),
                                    Err(denial) => Err(denial.clone()),
                                }
                            }),
                        };
                    (branch, frozen)
                })
                .collect();
        let branch_generations = branches
            .keys()
            .map(|branch| {
                (
                    branch.clone(),
                    state.branch_generations.get(branch).copied().unwrap_or(0),
                )
            })
            .collect();
        let cold = state.cold_naming.as_ref().and_then(|source| source.store());
        FrozenLineageNaming {
            branches,
            branch_generations,
            group_log: state.published_group_lineage.clone(),
            cold,
        }
    }
}

impl FrozenLineageNaming {
    /// The exact checkpoint store pinned when this capture was frozen, if any.
    pub fn cold_store(&self) -> Option<&PinnedNamingStore> {
        self.cold.as_ref()
    }

    /// Per-branch mutation generations observed when this capture was frozen.
    pub fn branch_generations(&self) -> &BTreeMap<BranchId, u64> {
        &self.branch_generations
    }

    /// Pages every branch (including any `required_branches` that hold a
    /// committed basis even before their first named lineage event) and the
    /// group log into `sink`.
    ///
    /// Complexity: O(branches + changes + rows of touched pages).
    pub fn seal(
        &self,
        required_branches: &[BranchId],
        sink: &mut dyn ChunkMemberSink,
    ) -> Result<CheckpointNamingState, Cause> {
        let mut all_branches: BTreeSet<BranchId> = self.branches.keys().cloned().collect();
        all_branches.extend(required_branches.iter().cloned());
        let mut roots = Vec::with_capacity(all_branches.len());
        let pinned_store = self.cold.as_ref().map(|pinned| pinned.store().as_ref());
        for branch in all_branches {
            let Some(frozen) = self.branches.get(&branch) else {
                roots.push(BranchNamingRoot {
                    branch,
                    naming: None,
                    index: None,
                    groups: None,
                    pages: Vec::new(),
                });
                continue;
            };
            let mut root = BranchNamingRoot {
                branch,
                naming: None,
                index: None,
                groups: None,
                pages: Vec::new(),
            };
            let unified = frozen.unified_base();
            let mut changes = Vec::new();
            if let Some(naming) = &frozen.naming {
                root.naming = Some(NamingStateHeader {
                    earliest_event_commit: naming
                        .as_ref()
                        .ok()
                        .and_then(|naming| naming.earliest_event_commit())
                        .map(|commit| commit.0),
                    latest_event_commit: naming
                        .as_ref()
                        .ok()
                        .and_then(|naming| naming.latest_event_commit())
                        .map(|commit| commit.0),
                    unavailable: naming.is_err(),
                });
                if let Ok(naming) = naming {
                    if unified.is_some() {
                        changes.extend(naming.encoded_changes());
                    } else {
                        changes.extend(naming.collect_encoded_in(pinned_store)?);
                    }
                }
            }
            if let Some(index) = &frozen.index {
                root.index = Some(match index {
                    BranchNameIndex::Current(index) => {
                        if unified.is_some() {
                            changes.extend(index.encoded_changes());
                        } else {
                            changes.extend(index.collect_encoded_in(pinned_store)?);
                        }
                        NameIndexHeader {
                            earliest_commit: index.earliest_commit().map(|commit| commit.0),
                            head: index.head().map(|commit| commit.0),
                            retention_floor: index.retention_floor().map(|commit| commit.0),
                            unavailable: false,
                        }
                    }
                    BranchNameIndex::Unavailable(_) => NameIndexHeader {
                        earliest_commit: None,
                        head: None,
                        retention_floor: None,
                        unavailable: true,
                    },
                });
            }
            if let Some(groups) = &frozen.groups {
                root.groups = Some(match groups {
                    Ok(groups) => {
                        for scheme in groups.schemes_raw() {
                            if unified.is_some() {
                                changes.extend(scheme.encoded_changes());
                            } else {
                                changes.extend(scheme.collect_encoded_in(pinned_store)?);
                            }
                        }
                        GroupStateHeader {
                            earliest_commit: groups.earliest_commit().map(|commit| commit.0),
                            latest_commit: groups.latest_commit().map(|commit| commit.0),
                            schemes: groups
                                .scheme_headers()
                                .map(|(id, rule)| SchemeHeader {
                                    id: id.to_owned(),
                                    rule: rule.canonical_tag(),
                                })
                                .collect(),
                            unavailable: false,
                        }
                    }
                    Err(_) => GroupStateHeader {
                        earliest_commit: None,
                        latest_commit: None,
                        schemes: Vec::new(),
                        unavailable: true,
                    },
                });
            }
            changes.sort_unstable_by(|left, right| left.0.cmp(&right.0));
            let prior = match unified {
                Some(Some(link)) => {
                    let store = self.cold.as_ref().ok_or(Cause::SourceUnavailable)?;
                    link.root_in(store)
                        .map(|prior_root| (store.store(), prior_root.pages.as_slice()))
                }
                Some(None) | None => None,
            };
            root.pages = seal_pages(
                prior.map(|(store, pages)| (store.as_ref(), pages)),
                changes,
                sink,
            )?;
            roots.push(root);
        }
        let (group_log, group_log_through) = self.seal_group_log(sink)?;
        Ok(CheckpointNamingState {
            branches: roots,
            group_log,
            group_log_through,
        })
    }

    fn seal_group_log(
        &self,
        sink: &mut dyn ChunkMemberSink,
    ) -> Result<
        (
            Vec<crate::lineage::naming_pages::NamingPageRef>,
            Option<u64>,
        ),
        Cause,
    > {
        use crate::lineage::naming_pages::codec::NamingCodec;
        let store = self.cold.as_ref();
        let through = store.and_then(|store| store.state().group_log_through);
        let mut next_ordinal: u64 = store
            .and_then(|store| {
                store
                    .state()
                    .group_log
                    .iter()
                    .rev()
                    .find(|p| p.last.first() == Some(&GROUP_LOG))
            })
            .map(|p| {
                <((u64, CommitId), [u8; 32])>::from_bytes(&p.last[1..]).map(|((ord, _), _)| ord)
            })
            .transpose()?
            .unwrap_or(0);
        let mut pending: Vec<_> = self
            .group_log
            .iter()
            .filter(|record| through.is_none_or(|t| record.commit.0 > t))
            .collect();
        pending.sort_unstable_by(|a, b| {
            a.commit
                .cmp(&b.commit)
                .then_with(|| branch_key_digest(&a.branch).cmp(&branch_key_digest(&b.branch)))
        });
        let mut changes = Vec::with_capacity(pending.len());
        let mut newest = through;
        for record in pending {
            next_ordinal = next_ordinal.checked_add(1).ok_or(Cause::Damaged)?;
            let frame = crate::durability::naming::encode_group_lineage_frame(&record.transitions)
                .map_err(|_| Cause::Damaged)?;
            let key = row_key(
                GROUP_LOG,
                &(
                    (next_ordinal, record.commit),
                    branch_key_digest(&record.branch),
                ),
            );
            changes.push((key, Some(frame)));
            newest = Some(newest.map_or(record.commit.0, |at| at.max(record.commit.0)));
        }
        let prior = store.map(|store| (store.store().as_ref(), store.state().group_log.as_slice()));
        Ok((seal_pages(prior, changes, sink)?, newest))
    }
}
