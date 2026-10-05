mod assembly;
mod coverage;
mod frontier;
mod graph;
mod logical_digest;
mod projection;
mod proof;
mod quarantine;
mod reclaim;
mod reclaim_release;
mod reuse_claims;
#[cfg(test)]
mod reuse_source_fixture;
mod row_index;
mod source_edge;
mod terminal;
#[cfg(test)]
mod tests;
use super::blob_record::{self, BlobFact, FrameKind};
use super::child_expectation::{ChildExpectation, ChildScope};
use super::journal_walk::SelectedCheckpointEvidence;
use super::record_walk::route_inventory::RouteInventory;
use super::{
    OfflineArtifactFamily, OfflineIndeterminatePhysicalReason, OfflineIntegrityObservationCounters,
    OfflineIntegrityOutcome as Outcome, OfflinePhysicalBlastRadius as Blast,
    OfflinePhysicalDamageCause as Cause,
};
use coverage::{Coverage, Unread};
use graph::{declaration, edge_names_child, uncertain_dependency, EdgePosition};
use row_index::RowIndex;
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
    /// The kind the record's route or its first bytes declare, when either
    /// says one. A row whose frame was read is of the kind of its fact.
    kind: Option<FrameKind>,
    fact: Option<BlobFact>,
    outcome: Outcome,
    route: Option<ExtentRoute>,
}

impl Selected {
    /// A row is reported under the family of its kind.
    fn family(&self) -> OfflineArtifactFamily {
        self.kind
            .map_or(OfflineArtifactFamily::Unrecognized, |kind| kind.family)
    }
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
    unread: Vec<Unread>,
    coverage: Coverage,
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
            unread: Vec::new(),
            coverage: Coverage::Complete,
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
        // An unread record gets no row here: a record that the source does
        // not answer for is unknown to every check that consults the source.
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
        self.admit_unread();
        let rows = RowIndex::new(&self.selected, &self.coverage);
        for row in rows.duplicates(&self.selected) {
            self.selected[row].outcome = damage(Cause::DuplicateIdentity);
        }
        graph::validate_chunks(&mut self.selected, &rows);
        reuse_claims::validate(&mut self.selected, &rows);
        let (records, sessions) = (&rows.records, &rows.sessions);
        let claims = frontier::ClaimIndex::new(
            &self.selected,
            &rows,
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
            let uncertain = uncertain_dependency(fact, &self.selected, &rows);
            if let Some(outcome) = uncertain {
                self.selected[index].outcome = outcome;
                continue;
            }
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
                    if declaration(&self.selected, sessions, *session)
                        .is_none_or(|(declared_store, _, _, _, _)| declared_store != store)
                    {
                        Some(Cause::ScopeMismatch)
                    } else {
                        let resolved = entries.iter().enumerate().all(|(position, edge)| {
                            let position = EdgePosition {
                                session,
                                kind: *kind,
                                level: *level,
                                index: node_index
                                    .checked_mul(4096)
                                    .and_then(|start| start.checked_add(position as u64)),
                            };
                            records
                                .get(&edge.record)
                                .and_then(|child| self.selected[*child].fact.as_ref())
                                .is_some_and(|child| edge_names_child(&position, edge, child))
                        });
                        (!resolved).then_some(Cause::Pointer)
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
                    let declared = declaration(&self.selected, sessions, *session).is_some_and(
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
                        records,
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
            }
        }
        terminal::validate(&mut self.selected, sessions, records, &self.checkpoint);
        reclaim::validate(&mut self.selected, &rows, self.selected_root_generation);
        reclaim_release::validate(
            &mut self.selected,
            &rows,
            self.historical_source.as_ref(),
            &self.routes,
        );
    }
}

fn damage(cause: Cause) -> Outcome {
    super::record_walk::damage(cause, None, Blast::Artifact)
}
