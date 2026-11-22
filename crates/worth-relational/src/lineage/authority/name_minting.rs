//! Allocation-independent persistent name minting authority and commit-boundary
//! collision guard.
//!
//! Minting hashes strictly canonical domain-tagged bytes of:
//! `(scheme_id, occurrence_key, role_path, sorted_parent_identities, canonical_discriminator)`
//! into a fixed 32-byte [`PersistentNameIdentity`]. It never observes native
//! allocation coordinates (`RecordId`, `EntityId`, `PartitionId`, `local_slot`,
//! `generation`, `LineageId`, `CommitId`, or worker order).

use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

use crate::lineage::data::{
    DiscriminatorBasisDenialReason, InstalledPersistentNameSchemeRegistry, LineageNameDenial,
    PersistentNameBirthParts, PersistentNameBirthRecord, PersistentNameBirthRequest,
    PersistentNameHolder, PersistentNameIdentity, PersistentNameReferenceBasis,
};

use super::discriminator_scan::extract_embedded_name_refs;

/// Domain separator prefix for persistent name identity derivation.
pub const PERSISTENT_NAME_DOMAIN_TAG: &[u8] = b"worth.relational.persistent-name.v1\0";

/// Relational authority that validates birth provenance, resolves referenced
/// discriminator and parent names, mints fixed-size persistent name identities,
/// and enforces live-holder and same-commit collision guards atomically before
/// commit seal.
pub struct PersistentNameMintingAuthority;

impl PersistentNameMintingAuthority {
    /// Validates and mints a commit batch of persistent name births.
    ///
    /// Complexity: O(B * (P log P + R log H + D)) where B is the number of
    /// births in the commit, P is each birth's parent count, R is referenced
    /// name count, D is discriminator byte length, and H is live-holder count.
    ///
    /// Fail-closed: if any birth fails validation, references an absent/unborn
    /// name or parent, or collides with an existing live holder (not retired in
    /// this commit) or another birth in the same commit, returns
    /// `Err(LineageNameDenial)` with zero emitted birth records.
    pub fn mint_commit_births(
        registry: &InstalledPersistentNameSchemeRegistry,
        live_holders: &BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
        retired_in_commit: &BTreeSet<PersistentNameIdentity>,
        requests: &[PersistentNameBirthRequest],
    ) -> Result<Vec<PersistentNameBirthRecord>, LineageNameDenial> {
        let mut records = Vec::with_capacity(requests.len());
        let mut commit_holders = BTreeMap::<PersistentNameIdentity, PersistentNameHolder>::new();
        // Tracks `(occurrence_key, role_path) -> set of minted identities`
        // earlier in this commit batch for SameOccurrence discriminator resolution.
        let mut minted_in_occurrence =
            BTreeMap::<(String, String), BTreeSet<PersistentNameIdentity>>::new();

        for request in requests {
            let scheme = registry.get(&request.scheme_id).ok_or_else(|| {
                LineageNameDenial::UnknownScheme {
                    scheme_id: request.scheme_id.clone(),
                }
            })?;

            if scheme.require_authored_occurrence() && request.occurrence_key.trim().is_empty() {
                return Err(LineageNameDenial::MissingAuthoredOccurrenceKey {
                    scheme_id: request.scheme_id.clone(),
                });
            }

            let role_rank = scheme.role_rank(&request.role_path).ok_or_else(|| {
                LineageNameDenial::UndeclaredRole {
                    scheme_id: request.scheme_id.clone(),
                    role_path: request.role_path.clone(),
                }
            })?;

            verify_discriminator_references(
                request,
                scheme,
                role_rank,
                live_holders,
                retired_in_commit,
                &minted_in_occurrence,
            )?;

            if !request.parent_identities.is_empty()
                && scheme.split_capable()
                && request.canonical_discriminator.is_empty()
            {
                return Err(LineageNameDenial::MissingSplitDiscriminator {
                    scheme_id: request.scheme_id.clone(),
                    role_path: request.role_path.clone(),
                });
            }

            // Canonicalize parent identities into ascending byte order, refuse
            // duplicates, and verify every parent exists at basis, retired set,
            // or an earlier birth in this commit.
            let mut sorted_parents = request.parent_identities.clone();
            sorted_parents.sort_unstable();
            if let Some(pair) = sorted_parents.windows(2).find(|pair| pair[0] == pair[1]) {
                return Err(LineageNameDenial::DuplicateParentIdentity {
                    scheme_id: request.scheme_id.clone(),
                    role_path: request.role_path.clone(),
                    parent: pair[0],
                });
            }
            for &parent in &sorted_parents {
                let exists = live_holders.contains_key(&parent)
                    || retired_in_commit.contains(&parent)
                    || commit_holders.contains_key(&parent);
                if !exists {
                    return Err(LineageNameDenial::UnknownParentIdentity {
                        scheme_id: request.scheme_id.clone(),
                        role_path: request.role_path.clone(),
                        parent,
                    });
                }
            }

            let identity = mint_identity_digest(
                &request.scheme_id,
                &request.occurrence_key,
                &request.role_path,
                &sorted_parents,
                &request.canonical_discriminator,
            );

            // Same-commit sibling collision guard.
            if let Some(first) = commit_holders.get(&identity) {
                return Err(LineageNameDenial::DuplicateBirth {
                    identity,
                    first: first.clone(),
                    second: request.holder.clone(),
                });
            }

            // Live-holder collision guard (excluding holders retired in this commit).
            if !retired_in_commit.contains(&identity)
                && let Some(first) = live_holders.get(&identity)
            {
                return Err(LineageNameDenial::DuplicateBirth {
                    identity,
                    first: first.clone(),
                    second: request.holder.clone(),
                });
            }

            commit_holders.insert(identity, request.holder.clone());
            minted_in_occurrence
                .entry((request.occurrence_key.clone(), request.role_path.clone()))
                .or_default()
                .insert(identity);
            let parts = PersistentNameBirthParts::new(
                request.scheme_id.clone(),
                request.occurrence_key.clone(),
                request.role_path.clone(),
                sorted_parents,
                request.canonical_discriminator.clone(),
            );
            records.push(PersistentNameBirthRecord::new(
                identity,
                scheme.subject_kind(),
                parts,
            ));
        }

        Ok(records)
    }
}

fn verify_discriminator_references(
    request: &PersistentNameBirthRequest,
    scheme: &crate::lineage::data::InstalledPersistentNameScheme,
    role_rank: u32,
    live_holders: &BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
    retired_in_commit: &BTreeSet<PersistentNameIdentity>,
    minted_in_occurrence: &BTreeMap<(String, String), BTreeSet<PersistentNameIdentity>>,
) -> Result<(), LineageNameDenial> {
    // 1. Scan canonical discriminator bytes so callers cannot smuggle raw
    //    `TAG_NAME_REF` digests without declaring them in `referenced_names`.
    let embedded = extract_embedded_name_refs(&request.canonical_discriminator).map_err(|()| {
        LineageNameDenial::InvalidDiscriminatorBasis {
            scheme_id: request.scheme_id.clone(),
            role_path: request.role_path.clone(),
            reason: DiscriminatorBasisDenialReason::UndeclaredEncodedNameReference {
                raw_digest: [0u8; 32],
            },
        }
    })?;
    let declared_digests: BTreeSet<[u8; 32]> = request
        .referenced_names
        .iter()
        .map(|entry| *entry.identity.as_bytes())
        .collect();
    for raw_digest in embedded {
        if !declared_digests.contains(&raw_digest) {
            return Err(LineageNameDenial::InvalidDiscriminatorBasis {
                scheme_id: request.scheme_id.clone(),
                role_path: request.role_path.clone(),
                reason: DiscriminatorBasisDenialReason::UndeclaredEncodedNameReference {
                    raw_digest,
                },
            });
        }
    }

    // 2. Verify every declared reference against authoritative live holders or
    //    earlier-role births minted in the same occurrence during this commit.
    for reference in &request.referenced_names {
        match &reference.basis {
            PersistentNameReferenceBasis::Basis => {
                let live_at_basis = live_holders.contains_key(&reference.identity)
                    && !retired_in_commit.contains(&reference.identity);
                if !live_at_basis {
                    return Err(LineageNameDenial::InvalidDiscriminatorBasis {
                        scheme_id: request.scheme_id.clone(),
                        role_path: request.role_path.clone(),
                        reason: DiscriminatorBasisDenialReason::AbsentBasisName {
                            identity: reference.identity,
                        },
                    });
                }
            }
            PersistentNameReferenceBasis::SameOccurrence { role } => {
                let referenced_rank = scheme.role_rank(role);
                if !referenced_rank.is_some_and(|rank| rank < role_rank) {
                    return Err(LineageNameDenial::InvalidDiscriminatorBasis {
                        scheme_id: request.scheme_id.clone(),
                        role_path: request.role_path.clone(),
                        reason: DiscriminatorBasisDenialReason::NonPrecedingRole {
                            referenced_role: role.clone(),
                        },
                    });
                }
                let minted_earlier = minted_in_occurrence
                    .get(&(request.occurrence_key.clone(), role.clone()))
                    .is_some_and(|set| set.contains(&reference.identity));
                if !minted_earlier {
                    return Err(LineageNameDenial::InvalidDiscriminatorBasis {
                        scheme_id: request.scheme_id.clone(),
                        role_path: request.role_path.clone(),
                        reason: DiscriminatorBasisDenialReason::UnbornSameOccurrenceName {
                            referenced_role: role.clone(),
                            identity: reference.identity,
                        },
                    });
                }
            }
        }
    }
    Ok(())
}

/// Computes the canonical 32-byte SHA-256 identity from sorted, domain-tagged
/// birth parts.
fn mint_identity_digest(
    scheme_id: &str,
    occurrence_key: &str,
    role_path: &str,
    sorted_parents: &[PersistentNameIdentity],
    canonical_discriminator: &[u8],
) -> PersistentNameIdentity {
    let mut hasher = Sha256::new();
    hasher.update(PERSISTENT_NAME_DOMAIN_TAG);
    write_len_prefixed(&mut hasher, scheme_id.as_bytes());
    write_len_prefixed(&mut hasher, occurrence_key.as_bytes());
    write_len_prefixed(&mut hasher, role_path.as_bytes());
    let parent_count = u64::try_from(sorted_parents.len()).expect("parent count fits u64");
    hasher.update(parent_count.to_be_bytes());
    for parent in sorted_parents {
        hasher.update(parent.as_bytes());
    }
    write_len_prefixed(&mut hasher, canonical_discriminator);
    PersistentNameIdentity::from_minted_digest(hasher.finalize().into())
}

fn write_len_prefixed(hasher: &mut Sha256, bytes: &[u8]) {
    let len = u64::try_from(bytes.len()).expect("part length fits u64");
    hasher.update(len.to_be_bytes());
    hasher.update(bytes);
}

pub(super) fn mint_entity_births(
    scheme_registry: &InstalledPersistentNameSchemeRegistry,
    live_holders: &BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
    retired_in_commit: &BTreeSet<PersistentNameIdentity>,
    created_entities: &BTreeSet<crate::identity::data::EntityId>,
    named_births: &BTreeMap<crate::identity::data::EntityId, PersistentNameBirthRequest>,
) -> Result<BTreeMap<crate::identity::data::EntityId, PersistentNameBirthRecord>, LineageNameDenial>
{
    if named_births.is_empty() {
        return Ok(BTreeMap::new());
    }
    for (entity_id, request) in named_births {
        if !created_entities.contains(entity_id) {
            return Err(LineageNameDenial::BirthWithoutCreate {
                scheme_id: request.scheme_id.clone(),
                occurrence_key: request.occurrence_key.clone(),
                role_path: request.role_path.clone(),
                holder: request.holder.clone(),
            });
        }
    }
    // Order by canonical meaning parts (scheme, occurrence, role topological
    // rank, role path, canonical discriminator) rather than `EntityId`
    // allocation order so same-occurrence dependencies mint in rank order and
    // `DuplicateBirth` `first`/`second` reporting is allocation-independent.
    let mut ordered: Vec<(crate::identity::data::EntityId, &PersistentNameBirthRequest)> =
        named_births.iter().map(|(&id, req)| (id, req)).collect();
    ordered.sort_by(|(_, a), (_, b)| {
        let rank_a = scheme_registry
            .get(&a.scheme_id)
            .and_then(|scheme| scheme.role_rank(&a.role_path))
            .unwrap_or(u32::MAX);
        let rank_b = scheme_registry
            .get(&b.scheme_id)
            .and_then(|scheme| scheme.role_rank(&b.role_path))
            .unwrap_or(u32::MAX);
        (
            &a.scheme_id,
            &a.occurrence_key,
            rank_a,
            &a.role_path,
            &a.canonical_discriminator,
            &a.holder,
        )
            .cmp(&(
                &b.scheme_id,
                &b.occurrence_key,
                rank_b,
                &b.role_path,
                &b.canonical_discriminator,
                &b.holder,
            ))
    });
    let entities: Vec<_> = ordered.iter().map(|(id, _)| *id).collect();
    let requests: Vec<_> = ordered.into_iter().map(|(_, req)| req.clone()).collect();
    let records = PersistentNameMintingAuthority::mint_commit_births(
        scheme_registry,
        live_holders,
        retired_in_commit,
        &requests,
    )?;
    Ok(entities.into_iter().zip(records).collect())
}
