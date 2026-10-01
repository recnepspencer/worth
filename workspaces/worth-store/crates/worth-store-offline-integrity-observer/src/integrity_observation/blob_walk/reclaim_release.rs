//! Released-generation source validation from independently observed selected
//! records. Historical source and WAL fate are separate evidence boundaries.

use std::collections::BTreeMap;

use super::super::record_walk::route_inventory::{RouteClass, RouteInventory};
use super::super::OfflineUnknownPhysicalReason as Unknown;
use super::{damage, BlobFact, Cause, HistoricalBlobSource, Outcome, Selected};

pub(super) fn validate(
    rows: &mut [Selected],
    records: &BTreeMap<[u8; 24], usize>,
    historical: Option<&HistoricalBlobSource>,
    routes: &RouteInventory,
) {
    for index in 0..rows.len() {
        if rows[index].outcome != Outcome::Intact {
            continue;
        }
        if matches!(
            rows[index].fact,
            Some(BlobFact::ReleasedDropSetManifest { .. })
        ) {
            rows[index].outcome = manifest_outcome(rows, records, historical, routes, index);
        }
    }
    for index in 0..rows.len() {
        if rows[index].outcome != Outcome::Intact {
            continue;
        }
        if matches!(
            rows[index].fact,
            Some(BlobFact::ReleasedReclaimDescriptor { .. })
        ) {
            rows[index].outcome = descriptor_outcome(rows, records, historical, routes, index);
        }
    }
}

fn manifest_outcome(
    rows: &[Selected],
    records: &BTreeMap<[u8; 24], usize>,
    historical: Option<&HistoricalBlobSource>,
    routes: &RouteInventory,
    index: usize,
) -> Outcome {
    if !route_inventory_intact(rows, routes) {
        return Outcome::Unknown(Unknown::ParentScopeUnavailable);
    }
    if historical.is_some_and(|source| !route_inventory_intact(&source.rows, &source.routes)) {
        return Outcome::Unknown(Unknown::ParentScopeUnavailable);
    }
    let Some(BlobFact::ReleasedDropSetManifest {
        store,
        object,
        session,
        generation,
        root,
        root_digest,
        publication_record,
        publication_digest,
        ..
    }) = rows[index].fact.as_ref()
    else {
        return damage(Cause::Framing);
    };
    let source = records
        .get(publication_record)
        .and_then(|row| rows.get(*row))
        .or_else(|| {
            historical
                .filter(|source| source.routes_intact)
                .and_then(|source| {
                    source
                        .rows
                        .iter()
                        .find(|row| row.record == *publication_record)
                })
        });
    // The embedded canonical frame pins identity, but is not evidence that
    // the named publication was selected before the first release batch.
    let Some(source) = source else {
        return Outcome::Unknown(Unknown::ParentScopeUnavailable);
    };
    if source.outcome != Outcome::Intact {
        return source.outcome.clone();
    }
    if matches!(source.fact.as_ref(), Some(BlobFact::Publication {
        store: found_store, object: found_object, session: found_session,
        generation: found_generation, root: found_root, root_digest: found_root_digest,
        frame_digest, ..
    }) if found_store == store && found_object == object
        && found_session == session && found_generation == generation
        && found_root == root && found_root_digest == root_digest
        && frame_digest == publication_digest)
    {
        Outcome::Intact
    } else {
        damage(Cause::Pointer)
    }
}

fn descriptor_outcome(
    rows: &[Selected],
    records: &BTreeMap<[u8; 24], usize>,
    historical: Option<&HistoricalBlobSource>,
    routes: &RouteInventory,
    index: usize,
) -> Outcome {
    if !route_inventory_intact(rows, routes) {
        return Outcome::Unknown(Unknown::ParentScopeUnavailable);
    }
    if historical.is_some_and(|source| !route_inventory_intact(&source.rows, &source.routes)) {
        return Outcome::Unknown(Unknown::ParentScopeUnavailable);
    }
    let Some(BlobFact::ReleasedReclaimDescriptor {
        store,
        attempt,
        basis_digest,
        manifest_record,
        manifest_digest,
        manifest_count,
        source_root,
        candidate_root,
        predecessor,
        cumulative_dropped,
        terminal: _,
        ..
    }) = rows[index].fact.as_ref()
    else {
        return damage(Cause::Framing);
    };
    if manifest_record == &rows[index].record || source_root.checked_add(1) != Some(*candidate_root)
    {
        return damage(Cause::Pointer);
    }
    if historical.is_none_or(|source| !source.routes_intact || source.generation != *source_root) {
        return Outcome::Unknown(Unknown::ParentScopeUnavailable);
    }
    let Some(manifest) = records.get(manifest_record).and_then(|row| rows.get(*row)) else {
        return damage(Cause::Pointer);
    };
    if manifest.outcome != Outcome::Intact {
        return manifest.outcome.clone();
    }
    let Some(BlobFact::ReleasedDropSetManifest {
        store: manifest_store,
        frame_digest,
        attempt: manifest_attempt,
        basis_digest: manifest_basis,
        dropped,
        ..
    }) = manifest.fact.as_ref()
    else {
        return damage(Cause::Pointer);
    };
    if store != manifest_store
        || attempt != manifest_attempt
        || basis_digest != manifest_basis
        || manifest_digest != frame_digest
        || usize::from(*manifest_count) != dropped.len()
        || dropped.iter().any(|record| records.contains_key(record))
    {
        return damage(Cause::Pointer);
    }
    if let Some((prior_record, prior_digest)) = predecessor {
        let Some(prior) = historical
            .and_then(|source| source.rows.iter().find(|row| row.record == *prior_record))
        else {
            return Outcome::Unknown(Unknown::ParentScopeUnavailable);
        };
        if prior.outcome != Outcome::Intact {
            return prior.outcome.clone();
        }
        let valid = matches!(prior.fact.as_ref(), Some(BlobFact::ReleasedReclaimDescriptor {
            store: prior_store, basis_digest: prior_basis, candidate_root: prior_root,
            cumulative_dropped: prior_count, terminal: false,
            frame_digest: found_digest, ..
        }) if prior_store == store && prior_basis == basis_digest
            && prior_root == source_root
            && found_digest == prior_digest
            && prior_count.checked_add(u64::from(*manifest_count)) == Some(*cumulative_dropped));
        if !valid
            || !historical.is_some_and(|source| {
                matches!(
                    source.routes.classes.get(prior_record),
                    Some(RouteClass::Blob(14 | 16))
                )
            })
        {
            return damage(Cause::Pointer);
        }
    } else {
        let Some(BlobFact::ReleasedDropSetManifest {
            publication_record,
            dropped,
            ..
        }) = manifest.fact.as_ref()
        else {
            return damage(Cause::Pointer);
        };
        if dropped.binary_search(publication_record).is_err() {
            return damage(Cause::Pointer);
        }
    }
    // Presence and matching hashes cannot independently establish WAL selection.
    Outcome::Unknown(Unknown::WalCoverageUnavailable)
}

fn route_inventory_intact(rows: &[Selected], routes: &RouteInventory) -> bool {
    routes.intact
        && routes.tier_intact()
        && routes.classes.iter().all(|(record, class)| match class {
            RouteClass::UnknownLegacy => false,
            RouteClass::Blob(kind) => rows.iter().any(|row| {
                row.record == *record
                    && row.outcome == Outcome::Intact
                    && row
                        .fact
                        .as_ref()
                        .is_some_and(|fact| fact.route_kind_matches(*kind))
            }),
            RouteClass::Opaque | RouteClass::DerivedDirectory | RouteClass::BTreeNode => true,
        })
}
