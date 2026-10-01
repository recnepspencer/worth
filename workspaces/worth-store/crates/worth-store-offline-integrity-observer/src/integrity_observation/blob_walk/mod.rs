use std::collections::BTreeMap;

mod assembly;
mod frontier;
mod graph;
mod logical_digest;
mod projection;
mod quarantine;
mod reclaim;
mod reclaim_release;
mod reuse_claims;
mod terminal;
#[cfg(test)]
mod tests;
use super::blob_record::{self, BlobFact};
use super::child_expectation::{ChildExpectation, ChildScope};
use super::journal_walk::SelectedCheckpointEvidence;
use super::record_walk::route_inventory::RouteInventory;
use super::{
    OfflineArtifactFamily, OfflineIndeterminatePhysicalReason, OfflineIntegrityObservationCounters,
    OfflineIntegrityOutcome as Outcome, OfflinePhysicalBlastRadius as Blast,
    OfflinePhysicalDamageCause as Cause,
};
use graph::{declaration, uncertain_dependency};
struct Pending {
    record: [u8; 24],
    logical_bytes: u64,
    path: String,
    generation: u64,
    bytes: Vec<u8>,
    route: ExtentRoute,
    interruption: Option<Outcome>,
}

struct ExtentRoute {
    format: [u8; 10],
    arena: u64,
    extent: u64,
    logical_bytes: u64,
    frames: Vec<(u64, u64)>,
}

struct Selected {
    record: [u8; 24],
    path: String,
    generation: u64,
    family: OfflineArtifactFamily,
    fact: Option<BlobFact>,
    outcome: Outcome,
    route: Option<ExtentRoute>,
}

pub(crate) struct HistoricalBlobSource {
    generation: u64,
    rows: Vec<Selected>,
    routes_intact: bool,
    routes: RouteInventory,
}

impl HistoricalBlobSource {
    pub(crate) fn with_routes(mut self, routes: RouteInventory) -> Self {
        self.routes = routes;
        self
    }
}

pub(crate) struct BlobRecordWalk {
    pending: Option<Pending>,
    selected: Vec<Selected>,
    maximum_graph_edges: u64,
    retained_graph_edges: u64,
    checkpoint: SelectedCheckpointEvidence,
    selected_root_generation: Option<u64>,
    historical_source: Option<HistoricalBlobSource>,
    routes: RouteInventory,
}

impl BlobRecordWalk {
    pub(crate) fn new(maximum_graph_edges: u64, checkpoint: SelectedCheckpointEvidence) -> Self {
        Self {
            pending: None,
            selected: Vec::new(),
            maximum_graph_edges,
            retained_graph_edges: 0,
            checkpoint,
            selected_root_generation: None,
            historical_source: None,
            routes: RouteInventory::new(),
        }
    }

    pub(crate) fn with_selected_root_generation(mut self, generation: Option<u64>) -> Self {
        self.selected_root_generation = generation;
        self
    }

    pub(crate) fn with_historical_source(mut self, source: HistoricalBlobSource) -> Self {
        self.historical_source = Some(source);
        self
    }

    pub(crate) fn with_route_inventory(mut self, routes: RouteInventory) -> Self {
        self.routes = routes;
        self
    }

    pub(crate) fn into_historical_source(
        mut self,
        generation: u64,
        routes_intact: bool,
    ) -> HistoricalBlobSource {
        if let Some(pending) = self.pending.take() {
            self.selected.push(projection::incomplete(pending, None));
        }
        HistoricalBlobSource {
            generation,
            rows: self.selected,
            routes_intact,
            routes: RouteInventory::new(),
        }
    }

    fn admit_fact(&mut self, fact: BlobFact) -> (Option<BlobFact>, Outcome) {
        // This is a distinct in-memory graph bound. Media entries_visited
        // remains the file/range traversal counter, while node edges retained
        // for the cross-record join never exceed the declared entry ceiling.
        let added_edges = match &fact {
            BlobFact::Node { entries, .. } => entries.len() as u64,
            BlobFact::DropSetManifest { dropped, .. } => dropped.len() as u64,
            BlobFact::ReleasedDropSetManifest { dropped, .. } => dropped.len() as u64,
            _ => 0,
        };
        if added_edges != 0 {
            let next = self.retained_graph_edges.saturating_add(added_edges);
            if next > self.maximum_graph_edges {
                return (
                    None,
                    Outcome::Indeterminate(OfflineIndeterminatePhysicalReason::EntryBoundExceeded),
                );
            }
            self.retained_graph_edges = next;
        }
        (Some(fact), Outcome::Intact)
    }

    fn validate_selected_claim_graph(&mut self) {
        let mut records: BTreeMap<[u8; 24], usize> = BTreeMap::new();
        let mut sessions: BTreeMap<[u8; 16], usize> = BTreeMap::new();
        let mut duplicate_rows = Vec::new();
        for (index, selected) in self.selected.iter().enumerate() {
            if let Some(prior) = records.get(&selected.record).copied() {
                if self.selected[prior].fact != selected.fact {
                    duplicate_rows.push(prior);
                    duplicate_rows.push(index);
                }
            } else {
                records.insert(selected.record, index);
            }
            if selected
                .fact
                .as_ref()
                .is_some_and(|fact| matches!(fact, BlobFact::Declaration { .. }))
            {
                let fact = selected.fact.as_ref().expect("checked declaration");
                let session = fact.session().expect("declaration has session");
                if let Some(prior) = sessions.get(&session).copied() {
                    if self.selected[prior].record != selected.record {
                        duplicate_rows.push(prior);
                        duplicate_rows.push(index);
                    }
                } else {
                    sessions.insert(session, index);
                }
            }
        }
        for index in duplicate_rows {
            self.selected[index].outcome = damage(Cause::DuplicateIdentity);
        }
        graph::validate_chunks(&mut self.selected, &sessions, &records);
        let reuse_claims = reuse_claims::validate(&mut self.selected, &sessions, &records);
        let claims = frontier::ClaimIndex::new(
            &self.selected,
            self.maximum_graph_edges
                .saturating_sub(self.retained_graph_edges),
        );
        for index in 0..self.selected.len() {
            if self.selected[index].outcome != Outcome::Intact {
                continue;
            }
            let Some(fact) = self.selected[index].fact.as_ref() else {
                continue;
            };
            let uncertain = uncertain_dependency(fact, &self.selected, &sessions, &records);
            if let Some(outcome) = uncertain {
                self.selected[index].outcome = outcome;
                continue;
            }
            let mut graph_uncertainty = None;
            let outcome = match fact {
                BlobFact::Declaration { .. }
                | BlobFact::Abandoned { .. }
                | BlobFact::DropSetManifest { .. }
                | BlobFact::ReleasedDropSetManifest { .. }
                | BlobFact::OriginalDropReserved { .. }
                | BlobFact::ReclaimDescriptor { .. }
                | BlobFact::ReleasedReclaimDescriptor { .. } => None,
                BlobFact::DedupeQuarantine { .. } => None,
                BlobFact::Chunk { .. } => None,
                BlobFact::ReuseClaim { .. } => None,
                BlobFact::Node {
                    store,
                    session,
                    kind,
                    level,
                    index: node_index,
                    entries,
                    ..
                } => {
                    if declaration(&self.selected, &sessions, *session)
                        .is_none_or(|(declared_store, _, _, _, _)| declared_store != store)
                    {
                        Some(Cause::ScopeMismatch)
                    } else {
                        let mut mismatch = None;
                        for (position, edge) in entries.iter().enumerate() {
                            let expected_index = node_index
                                .checked_mul(4096)
                                .and_then(|start| start.checked_add(position as u64));
                            let valid = records
                                .get(&edge.record)
                                .and_then(|target| self.selected[*target].fact.as_ref())
                                .is_some_and(|target| match (*kind, target) {
                                    (
                                        1,
                                        BlobFact::Chunk {
                                            digest,
                                            length,
                                            ordinal,
                                            session: child_session,
                                            ..
                                        },
                                    ) => {
                                        (*digest == edge.digest
                                            && *length == edge.covered
                                            && Some(*ordinal) == expected_index
                                            && *child_session == *session)
                                            || expected_index.is_some_and(|ordinal| {
                                                reuse_claims.get(&(*session, ordinal)).is_some_and(
                                                    |claim| match reuse_claims::selected_reuse_edge(
                                                        &self.selected[*claim],
                                                        edge,
                                                    ) {
                                                        Some(Outcome::Intact) => true,
                                                        Some(unknown @ Outcome::Unknown(_)) => {
                                                            graph_uncertainty = Some(unknown);
                                                            true
                                                        }
                                                        _ => false,
                                                    },
                                                )
                                            })
                                    }
                                    (
                                        2,
                                        BlobFact::Node {
                                            digest,
                                            covered,
                                            level: child_level,
                                            index: child_index,
                                            session: child_session,
                                            ..
                                        },
                                    ) => {
                                        *digest == edge.digest
                                            && *covered == edge.covered
                                            && child_level.checked_add(1) == Some(*level)
                                            && Some(*child_index) == expected_index
                                            && *child_session == *session
                                    }
                                    _ => false,
                                });
                            if !valid {
                                mismatch = Some(Cause::Pointer);
                                break;
                            }
                        }
                        mismatch
                    }
                }
                BlobFact::Publication {
                    store,
                    session,
                    object,
                    root,
                    root_digest,
                    total,
                    chunk_size,
                    scope,
                    ..
                } => {
                    let declared = declaration(&self.selected, &sessions, *session).is_some_and(
                        |(s, o, sc, size, bytes)| {
                            s == store
                                && o == object
                                && sc == scope
                                && size == chunk_size
                                && bytes == total
                        },
                    );
                    let root_valid = records
                        .get(root)
                        .and_then(|target| self.selected[*target].fact.as_ref())
                        .is_some_and(|target| {
                            matches!(target,
                            BlobFact::Node { frame_digest, covered, session: node_session, .. }
                            if frame_digest == root_digest && covered == total && node_session == session)
                        });
                    if !declared {
                        Some(Cause::ScopeMismatch)
                    } else if !root_valid {
                        Some(Cause::Pointer)
                    } else {
                        None
                    }
                }
                BlobFact::Frontier {
                    store,
                    session,
                    declaration_record,
                    declaration_digest,
                    next_chunk_ordinal,
                    durable_bytes,
                    last_chunk_record,
                    last_chunk_digest,
                } => {
                    let Some(claims) = claims.as_ref() else {
                        self.selected[index].outcome = Outcome::Indeterminate(
                            OfflineIndeterminatePhysicalReason::EntryBoundExceeded,
                        );
                        continue;
                    };
                    self.selected[index].outcome = frontier::validate(
                        &self.selected,
                        &records,
                        claims,
                        frontier::Claim {
                            store: *store,
                            session: *session,
                            declaration_record: *declaration_record,
                            declaration_digest: *declaration_digest,
                            next_chunk_ordinal: *next_chunk_ordinal,
                            durable_bytes: *durable_bytes,
                            last_chunk_record: *last_chunk_record,
                            last_chunk_digest: *last_chunk_digest,
                        },
                    );
                    continue;
                }
            };
            if let Some(cause) = outcome {
                self.selected[index].outcome = damage(cause);
            } else if let Some(unknown) = graph_uncertainty {
                self.selected[index].outcome = unknown;
            }
        }
        terminal::validate(&mut self.selected, &sessions, &records, &self.checkpoint);
        reclaim::validate(&mut self.selected, &records, self.selected_root_generation);
        reclaim_release::validate(
            &mut self.selected,
            &records,
            self.historical_source.as_ref(),
            &self.routes,
        );
    }
}

fn damage(cause: Cause) -> Outcome {
    super::record_walk::damage(cause, None, Blast::Artifact)
}
