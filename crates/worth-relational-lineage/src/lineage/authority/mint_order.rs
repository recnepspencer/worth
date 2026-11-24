//! The order a commit's pending births are minted in.
//!
//! A birth mints after every birth of the same commit it depends on: one its
//! discriminator references as born earlier in the same occurrence, and one
//! it lists as a parent. The schemes of the two do not matter, so a birth
//! may reference a name of a scheme whose identifier sorts after its own.
//!
//! Among births ready at the same time the canonical meaning order decides
//! (scheme, occurrence, role rank, role path, canonical discriminator,
//! holder), never native record allocation order. A commit with no
//! dependency against that order therefore mints exactly in it, and
//! `DuplicateBirth` `first`/`second` reporting stays allocation-independent.
//!
//! Births left on a dependency cycle are minted last, in canonical order.
//! Minting then refuses the first of them, whose reference is not yet born.

use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::lineage::data::{
    InstalledPersistentNameSchemeRegistry, LineageRecordTarget, PersistentNameBirthRequest,
    PersistentNameIdentity, PersistentNameReferenceBasis,
};

type Birth<'births> = (LineageRecordTarget, &'births PersistentNameBirthRequest);

/// Orders `named_births` so every birth follows its in-commit dependencies.
///
/// Complexity: O(B log B + D log B + bytes) for B births, D declared
/// dependencies and the birth parts each identity is hashed from.
pub(super) fn dependency_order<'births>(
    scheme_registry: &InstalledPersistentNameSchemeRegistry,
    named_births: &'births BTreeMap<LineageRecordTarget, PersistentNameBirthRequest>,
) -> Vec<Birth<'births>> {
    let canonical = canonical_order(scheme_registry, named_births);
    let mut minted_by = BTreeMap::<PersistentNameIdentity, Vec<usize>>::new();
    for (position, (_, request)) in canonical.iter().enumerate() {
        minted_by
            .entry(pending_identity(request))
            .or_default()
            .push(position);
    }
    // `waits[p]` counts the births `p` still waits on; `releases[d]` lists
    // the births waiting on `d`. Positions are canonical-order positions.
    let mut waits = vec![0usize; canonical.len()];
    let mut releases = vec![Vec::new(); canonical.len()];
    for (position, (_, request)) in canonical.iter().enumerate() {
        let dependencies: BTreeSet<usize> = in_commit_dependencies(request)
            .filter_map(|identity| minted_by.get(&identity))
            .flatten()
            .copied()
            .filter(|dependency| *dependency != position)
            .collect();
        waits[position] = dependencies.len();
        for dependency in dependencies {
            releases[dependency].push(position);
        }
    }
    let mut ready: BTreeSet<usize> = (0..canonical.len())
        .filter(|position| waits[*position] == 0)
        .collect();
    let mut order = Vec::with_capacity(canonical.len());
    while let Some(position) = ready.pop_first() {
        order.push(position);
        for &released in &releases[position] {
            waits[released] -= 1;
            if waits[released] == 0 {
                ready.insert(released);
            }
        }
    }
    let cyclic = (0..canonical.len()).filter(|position| waits[*position] != 0);
    order.extend(cyclic);
    order
        .into_iter()
        .map(|position| canonical[position])
        .collect()
}

/// The identities `request` needs born before it when the same commit bears
/// them: its same-occurrence discriminator references and its parents.
fn in_commit_dependencies(
    request: &PersistentNameBirthRequest,
) -> impl Iterator<Item = PersistentNameIdentity> + '_ {
    let referenced = request
        .referenced_names
        .iter()
        .filter(|reference| {
            matches!(
                reference.basis,
                PersistentNameReferenceBasis::SameOccurrence { .. }
            )
        })
        .map(|reference| reference.identity);
    referenced.chain(request.parent_identities.iter().copied())
}

fn pending_identity(request: &PersistentNameBirthRequest) -> PersistentNameIdentity {
    PersistentNameIdentity::of_pending_birth(
        &request.scheme_id,
        &request.occurrence_key,
        &request.role_path,
        &request.parent_identities,
        &request.canonical_discriminator,
    )
}

/// Births by canonical meaning parts, independent of record allocation.
fn canonical_order<'births>(
    scheme_registry: &InstalledPersistentNameSchemeRegistry,
    named_births: &'births BTreeMap<LineageRecordTarget, PersistentNameBirthRequest>,
) -> Vec<Birth<'births>> {
    let rank = |request: &PersistentNameBirthRequest| {
        scheme_registry
            .get(&request.scheme_id)
            .and_then(|scheme| scheme.role_rank(&request.role_path))
            .unwrap_or(u32::MAX)
    };
    let mut ordered: Vec<Birth<'births>> = named_births
        .iter()
        .map(|(&target, request)| (target, request))
        .collect();
    ordered.sort_by(|(_, a), (_, b)| {
        (
            &a.scheme_id,
            &a.occurrence_key,
            rank(a),
            &a.role_path,
            &a.canonical_discriminator,
            &a.holder,
        )
            .cmp(&(
                &b.scheme_id,
                &b.occurrence_key,
                rank(b),
                &b.role_path,
                &b.canonical_discriminator,
                &b.holder,
            ))
    });
    ordered
}
