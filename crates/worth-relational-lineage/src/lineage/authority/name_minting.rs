//! Allocation-independent persistent name minting authority and commit-boundary
//! collision guard.
//!
//! Minting hashes strictly canonical domain-tagged bytes of:
//! `(scheme_id, occurrence_key, role_path, sorted_parent_identities, canonical_discriminator)`
//! into a fixed 32-byte [`PersistentNameIdentity`]. It never observes native
//! allocation coordinates (`RecordId`, `EntityId`, `PartitionId`, `local_slot`,
//! `generation`, `LineageId`, `CommitId`, or worker order).

use crate::contracts::lineage::data::naming::{
    InstalledPersistentNameSchemeRegistry, PersistentNameBirthParts,
    PersistentNameBirthPartsOwnerPort as _, PersistentNameBirthRecord,
    PersistentNameBirthRecordOwnerPort as _, PersistentNameBirthRequest, PersistentNameHolder,
    PersistentNameIdentity, PersistentNameIdentityOwnerPort as _, PersistentNameReferenceBasis,
};
use std::collections::{BTreeMap, BTreeSet};

use crate::contracts::lineage::data::naming_denial::{
    DiscriminatorBasisDenialReason, LineageNameDenial,
};
use crate::contracts::lineage::data::postures::LineageBasisView;

use super::discriminator_scan::extract_embedded_name_refs;

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
    #[doc(hidden)]
    pub fn mint_commit_births_for_test(
        registry: &InstalledPersistentNameSchemeRegistry,
        live_holders: &BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
        retired_in_commit: &BTreeSet<PersistentNameIdentity>,
        requests: &[PersistentNameBirthRequest],
    ) -> Result<Vec<PersistentNameBirthRecord>, LineageNameDenial> {
        struct MapBasis<'a>(&'a BTreeMap<PersistentNameIdentity, PersistentNameHolder>);
        impl LineageBasisView for MapBasis<'_> {
            fn live_holder(&self, id: &PersistentNameIdentity) -> Option<&PersistentNameHolder> {
                self.0.get(id)
            }
            fn live_lineage(
                &self,
                _: &PersistentNameHolder,
            ) -> Option<crate::contracts::identity::data::primitives::LineageId> {
                None
            }
            fn retired_lineage(
                &self,
                _: &PersistentNameIdentity,
            ) -> Option<(
                crate::contracts::lineage::data::events::LineageSubject,
                crate::contracts::identity::data::primitives::LineageId,
            )> {
                None
            }
            fn holder_name(&self, _: &PersistentNameHolder) -> Option<PersistentNameIdentity> {
                None
            }
        }
        Self::mint_commit_births_with_basis(
            registry,
            &MapBasis(live_holders),
            retired_in_commit,
            requests,
        )
    }

    pub(super) fn mint_commit_births_with_basis(
        registry: &InstalledPersistentNameSchemeRegistry,
        basis: &dyn LineageBasisView,
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
                basis,
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
                let exists = basis.contains_live_holder(&parent)
                    || retired_in_commit.contains(&parent)
                    || basis.contains_retired(&parent)
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
                && let Some(first) = basis.live_holder(&identity)
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
    scheme: &crate::contracts::lineage::data::naming::InstalledPersistentNameScheme,
    role_rank: u32,
    basis: &dyn LineageBasisView,
    retired_in_commit: &BTreeSet<PersistentNameIdentity>,
    minted_in_occurrence: &BTreeMap<(String, String), BTreeSet<PersistentNameIdentity>>,
) -> Result<(), LineageNameDenial> {
    // 1. Scan canonical discriminator bytes so callers cannot smuggle raw
    //    `TAG_NAME_REF` digests without declaring them in `referenced_names`.
    let embedded = extract_embedded_name_refs(&request.canonical_discriminator).map_err(|()| {
        LineageNameDenial::InvalidDiscriminatorBasis {
            scheme_id: request.scheme_id.clone(),
            role_path: request.role_path.clone(),
            reason: DiscriminatorBasisDenialReason::MalformedDiscriminator,
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
                let live_at_basis = basis.contains_live_holder(&reference.identity)
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
    PersistentNameIdentity::from_minted_digest(canonical_birth_digest(
        scheme_id,
        occurrence_key,
        role_path,
        sorted_parents.iter().map(PersistentNameIdentity::as_bytes),
        canonical_discriminator,
    ))
}

/// The one canonical persistent-name hash: minting and persisted-birth
/// readmission both derive identity bytes only here.
///
/// Complexity: O(parts bytes + parents).
pub(super) use crate::contracts::lineage::data::birth_digest::canonical_birth_digest;

pub(super) fn mint_target_births(
    scheme_registry: &InstalledPersistentNameSchemeRegistry,
    basis: &dyn LineageBasisView,
    retired_in_commit: &BTreeSet<PersistentNameIdentity>,
    created_targets: &BTreeSet<crate::contracts::lineage::data::postures::LineageRecordTarget>,
    named_births: &BTreeMap<
        crate::contracts::lineage::data::postures::LineageRecordTarget,
        PersistentNameBirthRequest,
    >,
) -> Result<
    BTreeMap<
        crate::contracts::lineage::data::postures::LineageRecordTarget,
        PersistentNameBirthRecord,
    >,
    LineageNameDenial,
> {
    if named_births.is_empty() {
        return Ok(BTreeMap::new());
    }
    for (target, request) in named_births {
        if !created_targets.contains(target) {
            return Err(LineageNameDenial::BirthWithoutCreate {
                scheme_id: request.scheme_id.clone(),
                occurrence_key: request.occurrence_key.clone(),
                role_path: request.role_path.clone(),
                holder: request.holder.clone(),
            });
        }
    }
    // Order by canonical meaning parts (scheme, occurrence, role topological
    // rank, role path, canonical discriminator) rather than native record
    // allocation order so same-occurrence dependencies mint in rank order and
    // `DuplicateBirth` `first`/`second` reporting is allocation-independent.
    let mut ordered: Vec<(
        crate::contracts::lineage::data::postures::LineageRecordTarget,
        &PersistentNameBirthRequest,
    )> = named_births.iter().map(|(&id, req)| (id, req)).collect();
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
    let targets: Vec<_> = ordered.iter().map(|(id, _)| *id).collect();
    let requests: Vec<_> = ordered.into_iter().map(|(_, req)| req.clone()).collect();
    let records = PersistentNameMintingAuthority::mint_commit_births_with_basis(
        scheme_registry,
        basis,
        retired_in_commit,
        &requests,
    )?;
    Ok(targets.into_iter().zip(records).collect())
}
