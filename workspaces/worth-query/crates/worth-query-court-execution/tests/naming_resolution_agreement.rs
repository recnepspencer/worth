//! Cross-crate agreement court between `worth-query-declaration`'s
//! resolution vocabulary (`NameSelection`, `ResolvedName`,
//! `NameResolutionDenial`, `ErasedWorthQueryName`) and `worth-relational`'s
//! derived name index resolution contract, over real Relational committed name
//! minting and real group naming transitions.

use std::collections::{BTreeMap, BTreeSet};

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::{
    NameSelection, NameSelectionDenial, ResolvedName, WorthQueryName,
};
use worth_query_execution_naming::facade::{
    ExecutionNameBirthPlanBuilder, ExecutionNamingCommitAuthority, ExecutionPendingEntityBirth,
};
use worth_relational::facade::aggregate_identity::{
    AggregateContinuityRule, AggregateGroupMembers, AggregateGroupNamingAuthority,
    AggregateGroupScheme, AggregateGroupSchemeId, AggregateIdentity, AggregateMemberRole,
    AggregatePriorGroup,
};
use worth_relational::facade::history::{BranchId, CommitId};
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::lineage::{
    PersistentNameBirthRecord, PersistentNameBranchIndex, PersistentNameCommittedBatch,
    PersistentNameIdentity, PersistentNameKey, PersistentNameResolution,
    PersistentNameResolutionDenial, PersistentNameResolutionOutcome, PersistentNameSelection,
    PersistentNameTarget,
};
use worth_relational::facade::schema::NativeMemberKey;

use crate::naming_execution_fixture::{
    begin_staged_tx, create_face_intent, execution_lease_for_workers, face_ref, fresh_cad_runtime,
    lowered_cad_catalog, release_commit_snapshot, CadNamingSchema, Face, FaceSplitDisc,
    FACE_SCHEME_ID,
};

/// Commits four real `Face` entities through `ExecutionNamingCommitAuthority`,
/// each born on its own canonical `FaceSplitDisc::Side` discriminator.
fn commit_four_faces() -> Vec<(WorthQueryName<Face>, PersistentNameBirthRecord)> {
    let catalog = lowered_cad_catalog();
    let runtime = fresh_cad_runtime();
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);

    let refs: Vec<_> = (0..4u8).map(|i| face_ref(&format!("f-{i}"))).collect();
    let intents: Vec<_> = (0..4u8)
        .map(|i| create_face_intent(&format!("f-{i}")))
        .collect();
    let tx = begin_staged_tx(&runtime, "mint-four", intents);
    let mut builder = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    for (i, r) in refs.iter().enumerate() {
        builder
            .add_entity_birth(
                ExecutionPendingEntityBirth::new(
                    r.clone(),
                    FACE_SCHEME_ID,
                    "occ.court",
                    "authored",
                )
                .with_discriminator(&FaceSplitDisc::Side { side: i as u8 }),
            )
            .unwrap();
    }
    let committed = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &runtime,
        &execution,
        tx,
        builder.build(),
    )
    .expect("four distinct sides commit cleanly");
    let (res, admitted) = committed.into_parts();
    let out: Vec<_> = refs
        .iter()
        .map(|r| {
            let (eid, name) = admitted
                .created_entity_name::<CadNamingSchema, Face>(&res, r)
                .unwrap();
            let birth = admitted.entity_birth(eid).unwrap().clone();
            (name, birth)
        })
        .collect();
    release_commit_snapshot(&runtime, &res);
    out
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

#[derive(Debug, Eq, PartialEq)]
enum LiftedDenial {
    Ambiguous(Vec<[u8; 32]>),
    NoMatchingCandidate(Vec<[u8; 32]>),
    BasisPruned,
    UnknownName,
}

fn lift(
    outcome: PersistentNameResolutionOutcome,
) -> Result<ResolvedName<PersistentNameTarget>, LiftedDenial> {
    match outcome.result {
        Ok(PersistentNameResolution::Exact(target)) => Ok(ResolvedName::Exact(target)),
        Ok(PersistentNameResolution::Plural(targets)) => Ok(ResolvedName::Plural(targets)),
        Ok(PersistentNameResolution::Retired { at_commit }) => Ok(ResolvedName::Retired {
            at_commit: at_commit.0,
        }),
        Err(PersistentNameResolutionDenial::Ambiguous { candidates }) => Err(
            LiftedDenial::Ambiguous(candidates.into_iter().map(|k| *k.digest()).collect()),
        ),
        Err(PersistentNameResolutionDenial::NoMatchingCandidate { live_candidates }) => {
            Err(LiftedDenial::NoMatchingCandidate(
                live_candidates.into_iter().map(|k| *k.digest()).collect(),
            ))
        }
        Err(PersistentNameResolutionDenial::BasisPruned { .. }) => Err(LiftedDenial::BasisPruned),
        Err(PersistentNameResolutionDenial::UnknownName { .. }) => Err(LiftedDenial::UnknownName),
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
                            AggregateMemberRole::Interior,
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
                anchor: a.anchor.clone(),
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
            .find(|group| group.members.get(&member.into()).is_some())
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
    let faces = commit_four_faces();
    let minted: BTreeMap<[u8; 32], PersistentNameIdentity> = faces
        .iter()
        .map(|(_, birth)| (*birth.identity().as_bytes(), birth.identity()))
        .collect();
    let name_by_id: BTreeMap<PersistentNameIdentity, WorthQueryName<Face>> = faces
        .iter()
        .map(|(name, birth)| (birth.identity(), *name))
        .collect();

    // KeepSide carries exactly the canonical bytes Relational records at birth.
    let side = NameSelection::keep_side(&FaceSplitDisc::Side { side: 1 }).unwrap();
    let NameSelection::KeepSide { discriminator } = &side else {
        unreachable!()
    };
    assert_eq!(
        discriminator.as_bytes(),
        faces[1].1.parts().canonical_discriminator()
    );
    assert_ne!(
        discriminator.as_bytes(),
        faces[2].1.parts().canonical_discriminator()
    );
    assert_eq!(
        faces[0].0.erased().as_bytes(),
        PersistentNameKey::Entity(faces[0].1.identity()).digest()
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
        lift(index.resolve(held, CommitId(basis), &lower(selection, &minted)))
    };
    let mut fragments: Vec<[u8; 32]> = vec![*one.bytes(), *two.bytes()];
    fragments.sort();
    assert_eq!(
        resolve(&NameSelection::Exact, 2),
        Err(LiftedDenial::Ambiguous(fragments.clone()))
    );
    let Ok(ResolvedName::Plural(targets)) = resolve(&NameSelection::AllDescendants, 2) else {
        panic!("all descendants resolve plural");
    };
    let plural: Vec<[u8; 32]> = targets.iter().map(|t| *t.key().digest()).collect();
    assert_eq!(
        plural, fragments,
        "plural answers keep canonical name order"
    );
    let carriers = NameSelection::bounded_by_carriers([
        name_by_id[&sorted[3]].erased(),
        name_by_id[&sorted[2]].erased(),
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
        Err(LiftedDenial::NoMatchingCandidate(fragments))
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
    let at =
        |basis: u64| lift(index.resolve(held, CommitId(basis), &PersistentNameSelection::Exact));
    assert_eq!(
        at(3),
        Ok(ResolvedName::Exact(PersistentNameTarget::Group {
            group: two
        }))
    );
    assert_eq!(at(4), Ok(ResolvedName::Retired { at_commit: 4 }));
    let never = PersistentNameKey::Group(AggregateIdentity::from_bytes([9; 32]));
    assert_eq!(
        lift(index.resolve(never, CommitId(4), &PersistentNameSelection::Exact)),
        Err(LiftedDenial::UnknownName)
    );
    let pruned = groups.index(Some(CommitId(2)));
    assert_eq!(
        lift(pruned.resolve(held, CommitId(1), &PersistentNameSelection::Exact)),
        Err(LiftedDenial::BasisPruned)
    );
}
