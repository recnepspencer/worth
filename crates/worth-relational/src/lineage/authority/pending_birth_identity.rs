//! The identity a pending birth will be minted with, before its commit.
//!
//! A birth whose discriminator references a name born earlier in the same
//! occurrence and commit embeds that name's 32 bytes, so its author needs
//! them before commit finalization mints anything. This is the one door:
//! it runs the canonical persistent-name hash over typed birth parts and
//! never accepts raw identity bytes.
//!
//! The returned identity proves nothing by itself. Commit finalization still
//! requires every referenced identity to be a live holder at the basis or a
//! birth it minted earlier in the same occurrence, and refuses otherwise.

use crate::lineage::data::PersistentNameIdentity;

use super::name_minting::canonical_birth_digest;

impl PersistentNameIdentity {
    /// The identity commit finalization mints for a birth of exactly these
    /// parts. Parents may be given in any order.
    ///
    /// Complexity: O(parts bytes + P log P) for P parents.
    pub fn of_pending_birth(
        scheme_id: &str,
        occurrence_key: &str,
        role_path: &str,
        parent_identities: &[PersistentNameIdentity],
        canonical_discriminator: &[u8],
    ) -> Self {
        let mut sorted_parents = parent_identities.to_vec();
        sorted_parents.sort_unstable();
        Self::from_minted_digest(canonical_birth_digest(
            scheme_id,
            occurrence_key,
            role_path,
            sorted_parents.iter().map(PersistentNameIdentity::as_bytes),
            canonical_discriminator,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::lineage::authority::PersistentNameMintingAuthority;
    use crate::lineage::data::{
        InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry,
        PersistentNameBirthRequest, PersistentNameHolder, PersistentNameIdentity,
        PersistentNameSubjectKind,
    };

    fn registry() -> InstalledPersistentNameSchemeRegistry {
        let mut registry = InstalledPersistentNameSchemeRegistry::new();
        registry
            .register(InstalledPersistentNameScheme::from_ordered_roles(
                "worth.cad.face.v1",
                PersistentNameSubjectKind::Entity,
                ["authored", "cut_piece"],
                false,
                true,
            ))
            .expect("register face scheme");
        registry
    }

    fn request(
        role: &str,
        parents: Vec<PersistentNameIdentity>,
        slot: u64,
    ) -> PersistentNameBirthRequest {
        PersistentNameBirthRequest {
            scheme_id: "worth.cad.face.v1".to_owned(),
            occurrence_key: format!("occ.{slot}"),
            role_path: role.to_owned(),
            parent_identities: parents,
            referenced_names: Vec::new(),
            canonical_discriminator: Vec::new(),
            holder: PersistentNameHolder::Entity(crate::identity::data::EntityId::new(
                crate::identity::data::PartitionId::main(),
                slot,
                1,
            )),
        }
    }

    fn predicted(request: &PersistentNameBirthRequest) -> PersistentNameIdentity {
        PersistentNameIdentity::of_pending_birth(
            &request.scheme_id,
            &request.occurrence_key,
            &request.role_path,
            &request.parent_identities,
            &request.canonical_discriminator,
        )
    }

    /// The door and commit minting agree on a parentless birth and on a
    /// birth whose parents arrive in either order.
    #[test]
    fn a_pending_birth_identity_is_the_identity_minting_gives() {
        let registry = registry();
        let roots = [
            request("authored", vec![], 1),
            request("authored", vec![], 2),
        ];
        let minted_roots = PersistentNameMintingAuthority::mint_commit_births(
            &registry,
            &BTreeMap::new(),
            &BTreeSet::new(),
            &roots,
        )
        .expect("authored births mint");
        let mut live = BTreeMap::new();
        for (root, minted) in roots.iter().zip(&minted_roots) {
            assert_eq!(predicted(root), minted.identity());
            live.insert(minted.identity(), root.holder.clone());
        }
        let parents = [minted_roots[0].identity(), minted_roots[1].identity()];
        let forward = request("cut_piece", parents.to_vec(), 3);
        let reversed = request("cut_piece", vec![parents[1], parents[0]], 3);
        let minted = PersistentNameMintingAuthority::mint_commit_births(
            &registry,
            &live,
            &BTreeSet::new(),
            std::slice::from_ref(&forward),
        )
        .expect("a birth over live parents mints");
        assert_eq!(predicted(&forward), minted[0].identity());
        assert_eq!(predicted(&reversed), minted[0].identity());
        assert_ne!(predicted(&forward), predicted(&roots[0]));
    }
}
