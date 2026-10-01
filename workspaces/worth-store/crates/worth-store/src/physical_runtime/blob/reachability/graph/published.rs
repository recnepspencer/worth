use std::collections::BTreeSet;

use worth_store_physical_format::{BlobTreeNodeKind, PersistedRecordIdentity};

use super::super::scan::{closure::ClosureFact, SelectedInventory};
use super::super::BlobReachabilityFailure as Failure;

#[derive(Clone, Copy)]
enum ExpectedKind {
    Root,
    Tree(u8),
    Chunk,
}

#[derive(Clone, Copy)]
struct ExpectedEdge {
    record: PersistedRecordIdentity,
    digest: [u8; 32],
    covered_bytes: u64,
    start: u64,
    kind: ExpectedKind,
}

pub(super) fn authenticate(
    inventory: &SelectedInventory,
    live: &mut BTreeSet<PersistedRecordIdentity>,
    traversed: &mut u64,
) -> Result<(), Failure> {
    for (publication_record, fact) in inventory.facts.iter().filter(|(_, fact)| fact.current) {
        let Some(ClosureFact::Publication {
            store,
            session,
            root,
            root_digest,
            frame_digest: _,
            total_bytes,
            chunk_size,
            scope,
        }) = fact.closure.as_ref()
        else {
            continue;
        };
        let capacity =
            usize::try_from(inventory.maximum_edges()).map_err(|_| Failure::MetadataUnavailable)?;
        let mut stack = Vec::new();
        stack
            .try_reserve_exact(capacity)
            .map_err(|_| Failure::MetadataUnavailable)?;
        stack.push(ExpectedEdge {
            record: *root,
            digest: *root_digest,
            covered_bytes: *total_bytes,
            start: 0,
            kind: ExpectedKind::Root,
        });
        let mut expanded = BTreeSet::new();
        while let Some(edge) = stack.pop() {
            *traversed = traversed
                .checked_add(1)
                .ok_or(Failure::EdgeBoundExhausted)?;
            if *traversed > inventory.maximum_edges() {
                return Err(Failure::EdgeBoundExhausted);
            }
            let target = inventory
                .facts
                .get(&edge.record)
                .filter(|target| target.current)
                .ok_or(Failure::ConflictingSelectedFate)?;
            match (edge.kind, target.closure.as_ref()) {
                (
                    ExpectedKind::Root,
                    Some(ClosureFact::Tree {
                        store: found_store,
                        session: found_session,
                        covered_bytes,
                        frame_digest,
                        ..
                    }),
                ) if found_store == store
                    && found_session == session
                    && *covered_bytes == edge.covered_bytes
                    && *frame_digest == edge.digest => {}
                (
                    ExpectedKind::Tree(expected_level),
                    Some(ClosureFact::Tree {
                        store: found_store,
                        session: found_session,
                        level,
                        covered_bytes,
                        canonical_digest,
                        ..
                    }),
                ) if found_store == store
                    && found_session == session
                    && *level == expected_level
                    && *covered_bytes == edge.covered_bytes
                    && *canonical_digest == edge.digest => {}
                (
                    ExpectedKind::Chunk,
                    Some(ClosureFact::Chunk {
                        store: found_store,
                        session: found_session,
                        ordinal,
                        content_digest,
                        covered_bytes,
                        chunk_size: found_size,
                        scope: found_scope,
                    }),
                ) if found_store == store
                    && found_session == session
                    && *content_digest == edge.digest
                    && *covered_bytes == edge.covered_bytes
                    && *found_size == *chunk_size
                    && found_scope.is_none_or(|value| value == *scope)
                    && leaf_occurrence_matches(
                        *total_bytes,
                        *chunk_size,
                        edge.start,
                        edge.covered_bytes,
                        *ordinal,
                    ) => {}
                _ => return Err(Failure::ConflictingSelectedFate),
            }
            live.insert(edge.record);
            if !expanded.insert(edge.record) {
                continue;
            }
            let Some(ClosureFact::Tree {
                kind, level, links, ..
            }) = target.closure.as_ref()
            else {
                continue;
            };
            let mut child_start = edge.start;
            for link in links {
                let child_kind = match kind {
                    BlobTreeNodeKind::Leaf => ExpectedKind::Chunk,
                    BlobTreeNodeKind::Interior => ExpectedKind::Tree(
                        level
                            .checked_sub(1)
                            .ok_or(Failure::ConflictingSelectedFate)?,
                    ),
                };
                if stack.len() as u64 >= inventory.maximum_edges() {
                    return Err(Failure::EdgeBoundExhausted);
                }
                stack.push(ExpectedEdge {
                    record: link.record,
                    digest: link.digest,
                    covered_bytes: link.covered_bytes,
                    start: child_start,
                    kind: child_kind,
                });
                child_start = child_start
                    .checked_add(link.covered_bytes)
                    .ok_or(Failure::ConflictingSelectedFate)?;
            }
            if child_start
                != edge
                    .start
                    .checked_add(edge.covered_bytes)
                    .ok_or(Failure::ConflictingSelectedFate)?
            {
                return Err(Failure::ConflictingSelectedFate);
            }
        }
        live.insert(*publication_record);
    }
    Ok(())
}

// The same selected leaf-occurrence law used by released-closure authentication.
fn leaf_occurrence_matches(total: u64, size: u32, start: u64, covered: u64, ordinal: u64) -> bool {
    let size = u64::from(size);
    size != 0
        && start % size == 0
        && covered != 0
        && total
            .checked_sub(start)
            .map(|remaining| remaining.min(size))
            == Some(covered)
        && ordinal == start / size
}
