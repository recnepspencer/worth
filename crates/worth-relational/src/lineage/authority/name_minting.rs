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
    InstalledPersistentNameSchemeRegistry, LineageNameDenial, PersistentNameBirthParts,
    PersistentNameBirthRecord, PersistentNameBirthRequest, PersistentNameHolder,
    PersistentNameIdentity,
};

/// Domain separator prefix for persistent name identity derivation.
pub const PERSISTENT_NAME_DOMAIN_TAG: &[u8] = b"worth.relational.persistent-name.v1\0";

/// Relational authority that validates birth provenance, mints fixed-size
/// persistent name identities, and enforces live-holder and same-commit
/// collision guards atomically before commit seal.
pub struct PersistentNameMintingAuthority;

impl PersistentNameMintingAuthority {
    /// Validates and mints a commit batch of persistent name births.
    ///
    /// Complexity: O(B * (P log P + D)) where B is the number of births in the
    /// commit, P is each birth's parent count, and D is discriminator byte
    /// length, plus O(B log H) live-holder map probes.
    ///
    /// Fail-closed: if any birth fails validation or collides with an existing
    /// live holder (not retired in this commit) or another birth in the same
    /// commit, returns `Err(LineageNameDenial)` with zero emitted birth
    /// records.
    pub fn mint_commit_births(
        registry: &InstalledPersistentNameSchemeRegistry,
        live_holders: &BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
        retired_in_commit: &BTreeSet<PersistentNameIdentity>,
        requests: &[PersistentNameBirthRequest],
    ) -> Result<Vec<PersistentNameBirthRecord>, LineageNameDenial> {
        let mut records = Vec::with_capacity(requests.len());
        let mut commit_holders = BTreeMap::<PersistentNameIdentity, PersistentNameHolder>::new();

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

            // Every same-occurrence role referenced by the discriminator must
            // exist strictly earlier in the installed scheme's acyclic role DAG.
            for referenced in &request.referenced_same_occurrence_roles {
                let referenced_rank = scheme.role_rank(referenced);
                if !referenced_rank.is_some_and(|rank| rank < role_rank) {
                    return Err(LineageNameDenial::InvalidDiscriminatorBasis {
                        scheme_id: request.scheme_id.clone(),
                        role_path: request.role_path.clone(),
                        referenced_role: referenced.clone(),
                    });
                }
            }

            if !request.parent_identities.is_empty()
                && scheme.split_capable()
                && request.canonical_discriminator.is_empty()
            {
                return Err(LineageNameDenial::MissingSplitDiscriminator {
                    scheme_id: request.scheme_id.clone(),
                    role_path: request.role_path.clone(),
                });
            }

            // Canonicalize parent identities into ascending byte order and
            // refuse duplicate parent entries.
            let mut sorted_parents = request.parent_identities.clone();
            sorted_parents.sort_unstable();
            if let Some(pair) = sorted_parents.windows(2).find(|pair| pair[0] == pair[1]) {
                return Err(LineageNameDenial::DuplicateParentIdentity {
                    scheme_id: request.scheme_id.clone(),
                    role_path: request.role_path.clone(),
                    parent: pair[0],
                });
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
    named_births: &BTreeMap<crate::identity::data::EntityId, PersistentNameBirthRequest>,
) -> Result<BTreeMap<crate::identity::data::EntityId, PersistentNameBirthRecord>, LineageNameDenial>
{
    if named_births.is_empty() {
        return Ok(BTreeMap::new());
    }
    let entities: Vec<_> = named_births.keys().copied().collect();
    let requests: Vec<_> = named_births.values().cloned().collect();
    let records = PersistentNameMintingAuthority::mint_commit_births(
        scheme_registry,
        live_holders,
        retired_in_commit,
        &requests,
    )?;
    Ok(entities.into_iter().zip(records).collect())
}
