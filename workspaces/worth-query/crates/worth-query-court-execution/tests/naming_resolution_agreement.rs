//! Cross-crate agreement court between `worth-query-declaration`'s
//! resolution vocabulary (`NameSelection`, `ResolvedName`,
//! `NameResolutionDenial`, `ErasedWorthQueryName`) and `worth-relational`'s
//! derived name index resolution contract, over real Relational name minting
//! and real group naming transitions.

use std::collections::{BTreeMap, BTreeSet};

use worth_query_declaration::facade::application_naming::{
    ApplicationNameDiscriminator, NameResolutionDenial, NameSelection, NameSelectionDenial,
    ResolvedName, WorthQueryName,
};
use worth_query_declaration::persistent_name_admission_authority::admit_minted_digest;
use worth_relational::facade::aggregate_identity::{
    AggregateContinuityRule, AggregateGroupMembers, AggregateGroupNamingAuthority,
    AggregateGroupScheme, AggregateGroupSchemeId, AggregateIdentity, AggregateMemberRole,
    AggregatePriorGroup,
};
use worth_relational::facade::history::{BranchId, CommitId};
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::lineage::{
    InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry,
    PersistentNameBirthRecord, PersistentNameBirthRequest, PersistentNameBranchIndex,
    PersistentNameCommittedBatch, PersistentNameHolder, PersistentNameIdentity, PersistentNameKey,
    PersistentNameMintingAuthority, PersistentNameResolution, PersistentNameResolutionDenial,
    PersistentNameResolutionOutcome, PersistentNameSelection, PersistentNameSubjectKind,
    PersistentNameTarget,
};
use worth_relational::facade::schema::NativeMemberKey;

struct Space;
struct Face;

#[derive(Clone, Debug, Eq, PartialEq)]
enum FaceSide {
    Piece { index: u8 },
}

worth_query_declaration::worth_query_name_discriminator! {
    enum FaceSide {
        identity: "worth.court.face.side.v1",
        variants: {
            Piece { index },
        }
    }
}

/// Mints four faces through Relational's naming authority, each born on
/// its own canonical declaration side.
fn mint_faces() -> Vec<PersistentNameBirthRecord> {
    let mut registry = InstalledPersistentNameSchemeRegistry::new();
    registry
        .register(InstalledPersistentNameScheme::from_ordered_roles(
            "worth.court.face.v1",
            PersistentNameSubjectKind::Entity,
            ["authored"],
            true,
            true,
        ))
        .unwrap();
    let births: Vec<_> = (0..4u8)
        .map(|index| PersistentNameBirthRequest {
            scheme_id: "worth.court.face.v1".to_owned(),
            occurrence_key: "occ.court".to_owned(),
            role_path: "authored".to_owned(),
            parent_identities: Vec::new(),
            referenced_names: Vec::new(),
            canonical_discriminator: FaceSide::Piece { index }.canonical_bytes(),
            holder: PersistentNameHolder::PendingBirth {
                occurrence_key: "occ.court".to_owned(),
                role_path: "authored".to_owned(),
                ordinal: usize::from(index),
            },
        })
        .collect();
    PersistentNameMintingAuthority::mint_commit_births(
        &registry,
        &BTreeMap::new(),
        &BTreeSet::new(),
        &births,
    )
    .expect("distinct sides mint distinct faces")
}

/// Lowers a declaration selection onto Relational's selection contract.
fn lower(
    selection: &NameSelection,
    minted: &BTreeMap<[u8; 32], PersistentNameIdentity>,
) -> PersistentNameSelection {
    match selection {
        NameSelection::Exact => PersistentNameSelection::Exact,
        NameSelection::AllDescendants => PersistentNameSelection::AllDescendants,
        NameSelection::BoundedByCarriers { carriers } => {
            let lowered: BTreeSet<_> = carriers
                .as_slice()
                .iter()
                .map(|carrier| minted[carrier.as_bytes()])
                .collect();
            let declared: Vec<_> = carriers.as_slice().iter().map(|c| *c.as_bytes()).collect();
            let relational: Vec<_> = lowered.iter().map(|c| *c.as_bytes()).collect();
            assert_eq!(declared, relational, "both sides order carriers by digest");
            PersistentNameSelection::BoundedByCarriers { carriers: lowered }
        }
        NameSelection::KeepSide { discriminator } => PersistentNameSelection::KeepSide {
            discriminator: discriminator.as_bytes().to_vec(),
        },
    }
}

fn names<K>(keys: Vec<PersistentNameKey>) -> Vec<WorthQueryName<K>> {
    keys.iter()
        .map(|key| admit_minted_digest(*key.digest()))
        .collect()
}

/// Lifts a Relational outcome into the declaration resolution vocabulary.
fn lift<K>(
    outcome: PersistentNameResolutionOutcome,
) -> Result<ResolvedName<PersistentNameTarget>, NameResolutionDenial<K>> {
    match outcome.result {
        Ok(PersistentNameResolution::Exact(target)) => Ok(ResolvedName::Exact(target)),
        Ok(PersistentNameResolution::Plural(targets)) => Ok(ResolvedName::Plural(targets)),
        Ok(PersistentNameResolution::Retired { at_commit }) => Ok(ResolvedName::Retired {
            at_commit: at_commit.0,
        }),
        Err(PersistentNameResolutionDenial::Ambiguous { candidates }) => {
            Err(NameResolutionDenial::Ambiguous {
                candidates: names(candidates),
            })
        }
        Err(PersistentNameResolutionDenial::NoMatchingCandidate { live_candidates }) => {
            Err(NameResolutionDenial::NoMatchingCandidate {
                live_candidates: names(live_candidates),
            })
        }
        Err(PersistentNameResolutionDenial::BasisPruned { .. }) => {
            Err(NameResolutionDenial::BasisPruned)
        }
        Err(PersistentNameResolutionDenial::UnknownName { .. }) => {
            Err(NameResolutionDenial::UnknownName)
        }
        Err(PersistentNameResolutionDenial::EmptySelectionFilter) => {
            unreachable!("declaration selections never lower to an empty filter")
        }
    }
}

struct Groups {
    scheme: AggregateGroupScheme,
    prior: Vec<AggregatePriorGroup>,
    retired: BTreeSet<AggregateIdentity>,
    log: Vec<PersistentNameCommittedBatch>,
}

impl Groups {
    fn step(&mut self, partition: &[&[PersistentNameIdentity]]) {
        let proposed = partition
            .iter()
            .map(|group| {
                let mut members = AggregateGroupMembers::new();
                for name in group.iter() {
                    let mut slot = [0u8; 8];
                    slot.copy_from_slice(&name.as_bytes()[..8]);
                    let entity =
                        EntityId::new(PartitionId::main(), u64::from_be_bytes(slot) >> 8, 1);
                    members
                        .insert(
                            *name,
                            NativeMemberKey::Entity(entity),
                            AggregateMemberRole::Ordinary,
                        )
                        .unwrap();
                }
                members
            })
            .collect();
        let transition = AggregateGroupNamingAuthority::transition(
            &self.scheme,
            &self.prior,
            &self.retired,
            proposed,
        )
        .expect("valid group transition");
        self.retired.extend(transition.retired.iter().copied());
        self.prior = transition
            .assignments
            .iter()
            .map(|a| AggregatePriorGroup {
                identity: a.identity,
                anchor: a.anchor,
                sources: a.sources.clone(),
                members: a.members.clone(),
            })
            .collect();
        let commit = CommitId(self.log.len() as u64 + 1);
        self.log
            .push(PersistentNameCommittedBatch::new(commit).with_group_transition(&transition));
    }

    fn group_of(&self, member: PersistentNameIdentity) -> AggregateIdentity {
        self.prior
            .iter()
            .find(|group| group.members.get(&member).is_some())
            .unwrap()
            .identity
    }

    fn index(&self, floor: Option<CommitId>) -> PersistentNameBranchIndex {
        PersistentNameBranchIndex::rebuild_from_log(BranchId("main".to_owned()), floor, &self.log)
            .unwrap()
    }
}

#[test]
fn declaration_resolution_vocabulary_agrees_with_relational_name_index() {
    let faces = mint_faces();
    let minted: BTreeMap<[u8; 32], PersistentNameIdentity> = faces
        .iter()
        .map(|birth| (*birth.identity().as_bytes(), birth.identity()))
        .collect();
    // KeepSide carries exactly the canonical bytes Relational records at birth.
    let side = NameSelection::keep_side(&FaceSide::Piece { index: 1 }).unwrap();
    let NameSelection::KeepSide { discriminator } = &side else {
        unreachable!()
    };
    assert_eq!(
        discriminator.as_bytes(),
        faces[1].parts().canonical_discriminator()
    );
    assert_ne!(
        discriminator.as_bytes(),
        faces[2].parts().canonical_discriminator()
    );
    let face: WorthQueryName<Face> = admit_minted_digest(*faces[0].identity().as_bytes());
    assert_eq!(
        face.erased().as_bytes(),
        PersistentNameKey::Entity(faces[0].identity()).digest()
    );

    let sorted: Vec<_> = minted.values().copied().collect();
    let id = AggregateGroupSchemeId::new("worth.court.space.v1").unwrap();
    let mut groups = Groups {
        scheme: AggregateGroupScheme::new(id, AggregateContinuityRule::Anchor),
        prior: Vec::new(),
        retired: BTreeSet::new(),
        log: Vec::new(),
    };
    groups.step(&[&sorted]); // 1: one space
    let held_group = groups.group_of(sorted[0]);
    let held = PersistentNameKey::Group(held_group);
    // 2: the anchor member leaves, so the space retires into two fragments.
    groups.step(&[&sorted[1..2], &sorted[2..4]]);
    let (one, two) = (groups.group_of(sorted[1]), groups.group_of(sorted[2]));
    let index = groups.index(None);
    let resolve = |selection: &NameSelection, basis: u64| {
        lift::<Space>(index.resolve(held, CommitId(basis), &lower(selection, &minted)))
    };
    let mut fragments: Vec<WorthQueryName<Space>> = vec![
        admit_minted_digest(*one.bytes()),
        admit_minted_digest(*two.bytes()),
    ];
    fragments.sort();
    assert_eq!(
        resolve(&NameSelection::Exact, 2),
        Err(NameResolutionDenial::Ambiguous {
            candidates: fragments.clone()
        })
    );
    let Ok(ResolvedName::Plural(targets)) = resolve(&NameSelection::AllDescendants, 2) else {
        panic!("all descendants resolve plural");
    };
    let plural: Vec<WorthQueryName<Space>> = targets
        .iter()
        .map(|t| admit_minted_digest(*t.key().digest()))
        .collect();
    assert_eq!(
        plural, fragments,
        "plural answers keep canonical name order"
    );
    let carriers = NameSelection::bounded_by_carriers([
        admit_minted_digest::<Face>(*sorted[3].as_bytes()).erased(),
        admit_minted_digest::<Face>(*sorted[2].as_bytes()).erased(),
    ])
    .expect("two carriers form a nonempty filter");
    assert_eq!(
        NameSelection::bounded_by_carriers([]),
        Err(NameSelectionDenial::EmptyCarrierSet)
    );
    assert_eq!(
        resolve(&carriers, 2),
        Ok(ResolvedName::Exact(PersistentNameTarget::Group {
            group: two
        }))
    );
    assert_eq!(
        resolve(&side, 2),
        Err(NameResolutionDenial::NoMatchingCandidate {
            live_candidates: fragments
        })
    );
    assert_eq!(
        resolve(&NameSelection::Exact, 1),
        Ok(ResolvedName::Exact(PersistentNameTarget::Group {
            group: held_group
        }))
    );

    groups.step(&[&sorted[2..4]]); // 3: one fragment retires
    groups.step(&[]); // 4: the last fragment retires
    let index = groups.index(None);
    let at = |basis: u64| {
        lift::<Space>(index.resolve(held, CommitId(basis), &PersistentNameSelection::Exact))
    };
    assert_eq!(
        at(3),
        Ok(ResolvedName::Exact(PersistentNameTarget::Group {
            group: two
        }))
    );
    assert_eq!(at(4), Ok(ResolvedName::Retired { at_commit: 4 }));
    let never = PersistentNameKey::Group(AggregateIdentity::from_bytes([9; 32]));
    assert_eq!(
        lift::<Space>(index.resolve(never, CommitId(4), &PersistentNameSelection::Exact)),
        Err(NameResolutionDenial::UnknownName)
    );
    let pruned = groups.index(Some(CommitId(2)));
    assert_eq!(
        lift::<Space>(pruned.resolve(held, CommitId(1), &PersistentNameSelection::Exact)),
        Err(NameResolutionDenial::BasisPruned)
    );
}
