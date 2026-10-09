use std::collections::BTreeSet;

use worth_store_physical_format::{BlobTreeNodeKind, PersistedRecordIdentity};

use super::super::scan::{closure::ClosureFact, Role, SelectedInventory};
use super::super::BlobReachabilityFailure as Failure;

pub(super) fn authenticate(
    inventory: &SelectedInventory,
    live: &mut BTreeSet<PersistedRecordIdentity>,
    traversed: &mut u64,
) -> Result<(), Failure> {
    for (claim_record, fact) in inventory.facts.iter().filter(|(_, fact)| fact.current) {
        if !live.contains(claim_record) || !matches!(fact.role, Role::ReuseClaim { .. }) {
            continue;
        }
        let source = fact.reuse.ok_or(Failure::ConflictingSelectedFate)?;
        let publication = match inventory.facts.get(&source.publication_record) {
            Some(selected) if selected.current => {
                let Some(ClosureFact::Publication {
                    store,
                    session,
                    root,
                    root_digest,
                    frame_digest,
                    total_bytes,
                    chunk_size,
                    scope,
                }) = selected.closure.as_ref()
                else {
                    return Err(Failure::ConflictingSelectedFate);
                };
                if source
                    .witnessed_publication_digest
                    .is_some_and(|digest| digest != *frame_digest)
                    || source.witnessed_publication.is_some_and(|witness| {
                        witness.store() != *store
                            || witness.session() != *session
                            || witness.root_record() != *root
                            || witness.root_digest() != *root_digest
                            || witness.total_bytes() != *total_bytes
                            || witness.chunk_size() != *chunk_size
                            || witness.key_scope() != *scope
                    })
                {
                    return Err(Failure::ConflictingSelectedFate);
                }
                (
                    *store,
                    *session,
                    *root,
                    *root_digest,
                    *total_bytes,
                    *chunk_size,
                    *scope,
                )
            }
            _ => {
                let witness = source
                    .witnessed_publication
                    .ok_or(Failure::ConflictingSelectedFate)?;
                (
                    witness.store(),
                    witness.session(),
                    witness.root_record(),
                    witness.root_digest(),
                    witness.total_bytes(),
                    witness.chunk_size(),
                    witness.key_scope(),
                )
            }
        };
        let (store, session, mut node_record, mut expected_digest, total, size, scope) =
            publication;
        if source.store != store || source.scope != scope || source.chunk_size != size {
            return Err(Failure::ConflictingSelectedFate);
        }
        let mut offset = source
            .source_ordinal
            .checked_mul(u64::from(size))
            .filter(|offset| *offset < total)
            .ok_or(Failure::ConflictingSelectedFate)?;
        let expected_length = (total - offset).min(u64::from(size));
        if expected_length != u64::from(source.chunk_length) {
            return Err(Failure::ConflictingSelectedFate);
        }
        let mut expected_bytes = total;
        let mut expected_level = None;
        let mut found_chunk = false;
        let mut path = Vec::new();
        path.try_reserve_exact(7)
            .map_err(|_| Failure::MetadataUnavailable)?;
        for depth in 0..7 {
            step(inventory, traversed)?;
            let node = inventory
                .facts
                .get(&node_record)
                .filter(|fact| fact.current)
                .ok_or(Failure::ConflictingSelectedFate)?;
            let Some(ClosureFact::Tree {
                store: node_store,
                session: node_session,
                kind,
                level,
                covered_bytes,
                frame_digest,
                canonical_digest,
                links,
            }) = node.closure.as_ref()
            else {
                return Err(Failure::ConflictingSelectedFate);
            };
            let digest = if depth == 0 {
                frame_digest
            } else {
                canonical_digest
            };
            if *node_store != store
                || *node_session != session
                || *covered_bytes != expected_bytes
                || *digest != expected_digest
                || expected_level.is_some_and(|value| value != *level)
            {
                return Err(Failure::ConflictingSelectedFate);
            }
            path.push(node_record);
            let mut selected = None;
            for link in links {
                if offset < link.covered_bytes {
                    selected = Some(link);
                    break;
                }
                offset -= link.covered_bytes;
            }
            let link = selected.ok_or(Failure::ConflictingSelectedFate)?;
            if *kind == BlobTreeNodeKind::Leaf {
                if offset != 0
                    || link.record != source.selected_chunk
                    || link.digest != source.stored_digest
                    || link.covered_bytes != expected_length
                {
                    return Err(Failure::ConflictingSelectedFate);
                }
                found_chunk = true;
                break;
            }
            expected_level = Some(
                level
                    .checked_sub(1)
                    .ok_or(Failure::ConflictingSelectedFate)?,
            );
            node_record = link.record;
            expected_digest = link.digest;
            expected_bytes = link.covered_bytes;
        }
        if !found_chunk {
            return Err(Failure::ConflictingSelectedFate);
        }
        step(inventory, traversed)?;
        let chunk = inventory
            .facts
            .get(&source.selected_chunk)
            .filter(|fact| fact.current && matches!(fact.role, Role::Chunk { .. }))
            .ok_or(Failure::ConflictingSelectedFate)?;
        if !matches!(chunk.closure.as_ref(), Some(ClosureFact::Chunk {
            store: chunk_store, session: chunk_session, ordinal, content_digest,
            covered_bytes, chunk_size, ..
        }) if *chunk_store == store && *chunk_session == session
            && *ordinal == source.source_ordinal && *content_digest == source.stored_digest
            && *covered_bytes == expected_length && *chunk_size == size)
        {
            return Err(Failure::ConflictingSelectedFate);
        }
        // Commit reachability only after the entire selected source path and
        // occurrence have authenticated. Ordinary reads still need these
        // nodes after the source publication itself is released.
        live.extend(path);
        live.insert(source.selected_chunk);
    }
    Ok(())
}

fn step(inventory: &SelectedInventory, traversed: &mut u64) -> Result<(), Failure> {
    *traversed = traversed
        .checked_add(1)
        .filter(|count| *count <= inventory.maximum_edges())
        .ok_or(Failure::EdgeBoundExhausted)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use sha2::{Digest, Sha256};
    use worth_store_physical_format::{BlobGenerationPublicationV1, PersistedRecordIdentity};

    use super::*;
    use crate::physical_runtime::blob::reachability::{
        scan::{closure::TreeLink, reuse::ReuseSource, Fact},
        BlobReachabilityLimits,
    };

    fn id(ordinal: u64) -> PersistedRecordIdentity {
        PersistedRecordIdentity::new([9; 16], ordinal).unwrap()
    }

    fn fact(role: Role, closure: Option<ClosureFact>, reuse: Option<ReuseSource>) -> Fact {
        Fact {
            current: true,
            held: false,
            role,
            edges: vec![],
            declaration: None,
            claim: None,
            closure,
            control: None,
            reuse,
        }
    }

    #[test]
    fn selected_source_tree_edge_is_required_even_with_matching_chunk_bytes() {
        const CHUNK: u32 = 65_536;
        let limits = BlobReachabilityLimits::new(
            NonZeroU64::new(8).unwrap(),
            NonZeroU64::new(1024).unwrap(),
            NonZeroU64::new(2).unwrap(),
            NonZeroU64::new(16).unwrap(),
        );
        let mut inventory = SelectedInventory::new(limits).unwrap();
        let publication = BlobGenerationPublicationV1::new(
            [1; 16],
            [2; 16],
            [3; 16],
            1,
            id(2),
            [10; 32],
            u64::from(CHUNK),
            [4; 32],
            CHUNK,
            [5; 32],
        )
        .unwrap();
        let frame_digest: [u8; 32] = Sha256::digest(publication.encode()).into();
        inventory.facts.insert(
            id(1),
            fact(
                Role::Publication([2; 16]),
                Some(ClosureFact::Publication {
                    store: [1; 16],
                    session: [2; 16],
                    root: id(2),
                    root_digest: [10; 32],
                    frame_digest,
                    total_bytes: u64::from(CHUNK),
                    chunk_size: CHUNK,
                    scope: [5; 32],
                }),
                None,
            ),
        );
        inventory.facts.insert(
            id(2),
            fact(
                Role::Tree([2; 16]),
                Some(ClosureFact::Tree {
                    store: [1; 16],
                    session: [2; 16],
                    kind: BlobTreeNodeKind::Leaf,
                    level: 0,
                    covered_bytes: u64::from(CHUNK),
                    frame_digest: [10; 32],
                    canonical_digest: [11; 32],
                    links: vec![TreeLink {
                        record: id(3),
                        digest: [12; 32],
                        covered_bytes: u64::from(CHUNK),
                    }],
                }),
                None,
            ),
        );
        inventory.facts.insert(
            id(3),
            fact(
                Role::Chunk {
                    session: [2; 16],
                    ordinal: 0,
                },
                Some(ClosureFact::Chunk {
                    store: [1; 16],
                    session: [2; 16],
                    ordinal: 0,
                    content_digest: [12; 32],
                    covered_bytes: u64::from(CHUNK),
                    chunk_size: CHUNK,
                    scope: None,
                }),
                None,
            ),
        );
        let source = ReuseSource {
            publication_record: id(1),
            selected_chunk: id(3),
            source_ordinal: 0,
            store: [1; 16],
            scope: [5; 32],
            chunk_size: CHUNK,
            chunk_length: CHUNK,
            stored_digest: [12; 32],
            witnessed_publication: None,
            witnessed_publication_digest: None,
        };
        inventory.facts.insert(
            id(4),
            fact(
                Role::ReuseClaim {
                    session: [6; 16],
                    ordinal: 0,
                },
                None,
                Some(source),
            ),
        );
        let mut live = BTreeSet::from([id(4)]);
        let mut traversed = 0;
        authenticate(&inventory, &mut live, &mut traversed).unwrap();
        assert!(live.contains(&id(3)));
        assert!(
            live.contains(&id(2)),
            "authenticated source root is read-critical"
        );

        inventory.facts.remove(&id(3));
        assert!(matches!(
            authenticate(&inventory, &mut BTreeSet::from([id(4)]), &mut 0),
            Err(Failure::ConflictingSelectedFate)
        ));
        inventory.facts.insert(
            id(3),
            fact(
                Role::Chunk {
                    session: [2; 16],
                    ordinal: 0,
                },
                Some(ClosureFact::Chunk {
                    store: [1; 16],
                    session: [2; 16],
                    ordinal: 0,
                    content_digest: [12; 32],
                    covered_bytes: u64::from(CHUNK),
                    chunk_size: CHUNK,
                    scope: None,
                }),
                None,
            ),
        );
        if let Some(ClosureFact::Tree { links, .. }) =
            &mut inventory.facts.get_mut(&id(2)).unwrap().closure
        {
            links[0].digest = [99; 32];
        }
        assert!(matches!(
            authenticate(&inventory, &mut BTreeSet::from([id(4)]), &mut 0),
            Err(Failure::ConflictingSelectedFate)
        ));
        if let Some(ClosureFact::Tree { links, .. }) =
            &mut inventory.facts.get_mut(&id(2)).unwrap().closure
        {
            links[0].digest = [12; 32];
        }
        inventory.facts.remove(&id(1));
        assert!(
            matches!(
                authenticate(&inventory, &mut BTreeSet::from([id(4)]), &mut 0),
                Err(Failure::ConflictingSelectedFate)
            ),
            "V1 cannot invent a missing publication"
        );
        inventory.facts.get_mut(&id(4)).unwrap().reuse = Some(ReuseSource {
            witnessed_publication: Some(publication),
            witnessed_publication_digest: Some(frame_digest),
            ..source
        });
        let mut live = BTreeSet::from([id(4)]);
        authenticate(&inventory, &mut live, &mut 0).unwrap();
        assert!(
            live.contains(&id(3)),
            "selected V2 witness supports released source"
        );
        assert!(live.contains(&id(2)), "V2 retains its selected source path");
    }
}
