//! Slice 5D same-commit discriminator references through operation-output
//! lowering and Relational's commit finalization.
//!
//! A later-role output of one occurrence tells itself apart by an output the
//! same program named earlier. The courts' oracle is Relational itself: a
//! commit succeeds only when the name lowering wrote into the discriminator
//! is the name Relational minted for the referenced birth, and the resulting
//! name must equal the one the ordinary two-commit route gives.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::{
    WorthQueryName, WorthQuerySameCommitName,
};
use worth_query_execution_naming::facade::{
    same_commit_name_of, BoundOperationNaming, NameReadDependencies, OperationMergeKeeper,
    OperationNamedOutput, OperationNamingEntry, WorthQueryOutputBirth, WorthQueryOutputNameDenial,
};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::lineage::{
    CommitNamingPlan, PendingBirthTarget, PersistentNameBirthRecord, PersistentNameHolder,
};
use worth_relational::facade::transactions::{
    CommitResult, DeleteEntityIntent, EntityMutationIntent, MutationIntent,
};

use crate::naming_execution_fixture::{
    begin_staged_tx, create_face_intent, execution_lease_for_workers, face_ref,
    lowered_cad_catalog, release_commit_snapshot, CadNamingSchema, Face, FaceSplitDisc,
    FACE_SCHEME_ID,
};
use crate::naming_tracked_world::{NamedFace, TrackedWorld};

const ENTITY: worth_query_declaration::facade::application_naming::ApplicationNameSubjectKind =
    worth_query_declaration::facade::application_naming::ApplicationNameSubjectKind::Entity;

/// Which patch of a cut piece this is, the piece born by the same commit.
pub struct PatchOfFresh {
    pub piece: WorthQuerySameCommitName<Face>,
    pub index: u8,
}
worth_query_declaration::worth_query_name_discriminator! {
    struct PatchOfFresh {
        identity: "worth.cad.face.patch-of.v1",
        fields: [piece, index]
    }
}

/// The same discriminator over a piece an earlier commit bore.
struct PatchOfHeld {
    piece: WorthQueryName<Face>,
    index: u8,
}
worth_query_declaration::worth_query_name_discriminator! {
    struct PatchOfHeld {
        identity: "worth.cad.face.patch-of.v1",
        fields: [piece, index]
    }
}

pub fn side(occurrence: &str, role: &str, side: u8) -> WorthQueryOutputBirth {
    WorthQueryOutputBirth::new(occurrence, role).with_discriminator(&FaceSplitDisc::Side { side })
}

/// One program under authoring, as a candidate writer builds it.
pub struct Authoring {
    pub program: BoundOperationNaming,
}

impl Authoring {
    pub fn new() -> Self {
        Self {
            program: BoundOperationNaming::default(),
        }
    }

    pub fn output(
        key: &str,
        birth: WorthQueryOutputBirth,
    ) -> OperationNamedOutput<PendingBirthTarget> {
        OperationNamedOutput {
            target: PendingBirthTarget::CreatedEntity(face_ref(key)),
            scheme_id: FACE_SCHEME_ID.to_owned(),
            birth,
        }
    }

    pub fn birth(&mut self, key: &str, birth: WorthQueryOutputBirth) -> &mut Self {
        self.program
            .push(OperationNamingEntry::Birth(Self::output(key, birth)));
        self
    }

    /// The reference to the name the already named face `key` is born with.
    pub fn fresh(&self, key: &str) -> WorthQuerySameCommitName<Face> {
        let target = PendingBirthTarget::CreatedEntity(face_ref(key));
        same_commit_name_of(&self.program, &target).expect("the program names the referenced face")
    }

    pub fn patch(&self, occurrence: &str, of: &str, index: u8) -> WorthQueryOutputBirth {
        WorthQueryOutputBirth::new(occurrence, "seam_patch").with_discriminator(&PatchOfFresh {
            piece: self.fresh(of),
            index,
        })
    }
}

/// A world with one committed face: lowering reads a non-empty branch.
pub fn seeded_world(execution: &ExecutionRequest<'_, '_>) -> TrackedWorld {
    let world = TrackedWorld::new();
    world.seed(execution, "seed", &["seeded"]);
    world
}

pub fn lower(
    world: &TrackedWorld,
    execution: &ExecutionRequest<'_, '_>,
    program: &BoundOperationNaming,
) -> Result<CommitNamingPlan, WorthQueryOutputNameDenial> {
    let head = world.head();
    world
        .runtime
        .with_persistent_names_at_observation(execution, &head, |view| {
            program.lower(&lowered_cad_catalog(), view)
        })
        .expect("name index readable")
}

/// Creates `creates` in order, deletes `deletes`, and commits `plan`.
fn commit(
    world: &TrackedWorld,
    execution: &ExecutionRequest<'_, '_>,
    creates: &[&str],
    deletes: &[EntityId],
    plan: CommitNamingPlan,
) -> CommitResult {
    let mut intents: Vec<_> = deletes
        .iter()
        .map(|entity_id| {
            MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
                entity_id: *entity_id,
            }))
        })
        .collect();
    intents.extend(creates.iter().map(|key| create_face_intent(key)));
    let mut tx = begin_staged_tx(&world.runtime, "same-commit", intents);
    tx.attach_naming_plan(plan);
    let result = world
        .runtime
        .commit_branch_transaction(execution, tx)
        .expect("Relational mints exactly the names lowering referenced");
    release_commit_snapshot(&world.runtime, &result);
    result
}

/// The one birth `result` committed under `role` with discriminator `index`.
fn born(result: &CommitResult, role: &str, index: u8) -> (EntityId, PersistentNameBirthRecord) {
    let mut births = result
        .committed_name_births()
        .into_iter()
        .filter(|(_, birth)| {
            birth.parts().role_path() == role
                && birth.parts().canonical_discriminator().last() == Some(&index)
        });
    let (holder, birth) = births.next().expect("the role was born");
    assert!(births.next().is_none(), "one birth per role and index");
    match holder {
        PersistentNameHolder::Entity(entity) => (entity, birth),
        other => panic!("faces are entities: {other:?}"),
    }
}

fn embeds(birth: &PersistentNameBirthRecord, referenced: &PersistentNameBirthRecord) -> bool {
    let digest = referenced.identity();
    birth
        .parts()
        .canonical_discriminator()
        .windows(32)
        .any(|window| window == digest.as_bytes())
}

/// Cuts a piece and patches it in one commit, creating the faces in
/// `creates` order beside an unrelated birth declared `unrelated_first`.
fn cut_and_patch_in_one_commit(
    execution: &ExecutionRequest<'_, '_>,
    creates: &[&str],
    unrelated_first: bool,
) -> (PersistentNameBirthRecord, PersistentNameBirthRecord) {
    let world = seeded_world(execution);
    let mut authoring = Authoring::new();
    if unrelated_first {
        authoring.birth("other", side("occ.other", "authored", 9));
    }
    authoring.birth("cut", side("occ.cut", "cut_piece", 1));
    let patch = authoring.patch("occ.cut", "cut", 7);
    authoring.birth("patch", patch);
    if !unrelated_first {
        authoring.birth("other", side("occ.other", "authored", 9));
    }
    let plan = lower(&world, execution, &authoring.program).expect("the program lowers");
    let result = commit(&world, execution, creates, &[], plan);
    (
        born(&result, "cut_piece", 1).1,
        born(&result, "seam_patch", 7).1,
    )
}

#[test]
fn naming_same_commit_reference_gives_the_name_the_two_commit_route_gives() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let (cut, patch) = cut_and_patch_in_one_commit(&execution, &["cut", "patch", "other"], false);
    assert!(embeds(&patch, &cut), "the patch names its piece");
    assert!(
        !patch
            .parts()
            .canonical_discriminator()
            .windows(32)
            .any(|window| window == [0u8; 32]),
        "no placeholder survives lowering"
    );

    // Independent creation order and declaration order never change a name.
    let permuted = cut_and_patch_in_one_commit(&execution, &["other", "patch", "cut"], true);
    assert_eq!(permuted.0.identity(), cut.identity());
    assert_eq!(permuted.1.identity(), patch.identity());

    // The ordinary route: the piece is born first, and a later commit's
    // patch embeds its committed name. Both routes mint the same bytes.
    let world = seeded_world(&execution);
    let mut first = Authoring::new();
    first.birth("cut", side("occ.cut", "cut_piece", 1));
    let plan = lower(&world, &execution, &first.program).unwrap();
    let committed = commit(&world, &execution, &["cut"], &[], plan);
    let (cut_entity, earlier_cut) = born(&committed, "cut_piece", 1);
    let head = world.head();
    let held = world
        .reader(&execution, &head)
        .entity_name_of::<CadNamingSchema, Face>(cut_entity, &mut NameReadDependencies::new())
        .expect("the piece carries its committed name");
    let mut second = Authoring::new();
    second.birth(
        "patch",
        WorthQueryOutputBirth::new("occ.cut", "seam_patch").with_discriminator(&PatchOfHeld {
            piece: held,
            index: 7,
        }),
    );
    let plan = lower(&world, &execution, &second.program).unwrap();
    let later_patch = born(
        &commit(&world, &execution, &["patch"], &[], plan),
        "seam_patch",
        7,
    )
    .1;
    assert_eq!(earlier_cut.identity(), cut.identity());
    assert_eq!(later_patch.identity(), patch.identity());
    assert_eq!(
        later_patch.parts().canonical_discriminator(),
        patch.parts().canonical_discriminator()
    );
}

fn split_into(
    parent: NamedFace,
    children: Vec<OperationNamedOutput<PendingBirthTarget>>,
) -> OperationNamingEntry<PendingBirthTarget> {
    OperationNamingEntry::Split {
        subject: ENTITY,
        parent: *parent.name.as_bytes(),
        children,
    }
}

fn merge_minting(
    parents: [NamedFace; 2],
    survivor: OperationNamedOutput<PendingBirthTarget>,
) -> OperationNamingEntry<PendingBirthTarget> {
    OperationNamingEntry::Merge {
        subject: ENTITY,
        parents: parents
            .iter()
            .map(|parent| *parent.name.as_bytes())
            .collect(),
        keeper: OperationMergeKeeper::MintNew(survivor),
    }
}

#[test]
fn naming_same_commit_references_reach_and_leave_split_children_and_merge_survivors() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::new();
    let (seeded, _) = world.seed(&execution, "seed", &["p", "q", "m1", "m2", "n1", "n2"]);
    let [p, q, m1, m2, n1, n2] = seeded[..] else {
        panic!("six seeded faces")
    };
    let occurrence = "occ.sculpt";
    let cut = |index| side(occurrence, "cut_piece", index);
    let mut authoring = Authoring::new();
    // Split children and a minted merge survivor are born under `cut_piece`.
    authoring.program.push(split_into(
        p,
        vec![
            Authoring::output("a1", cut(1)),
            Authoring::output("a2", cut(2)),
        ],
    ));
    authoring
        .program
        .push(merge_minting([m1, m2], Authoring::output("mm", cut(3))));
    // A plain birth, both children of a second split and a second minted
    // survivor each reference one of them.
    let (of_a1, of_mm) = (
        authoring.patch(occurrence, "a1", 11),
        authoring.patch(occurrence, "mm", 13),
    );
    authoring.birth("b", of_a1).birth("b2", of_mm);
    let children = vec![
        Authoring::output("c1", authoring.patch(occurrence, "a1", 14)),
        Authoring::output("c2", authoring.patch(occurrence, "a2", 15)),
    ];
    authoring.program.push(split_into(q, children));
    let survivor = Authoring::output("nn", authoring.patch(occurrence, "a2", 16));
    authoring.program.push(merge_minting([n1, n2], survivor));

    let plan = lower(&world, &execution, &authoring.program).expect("the program lowers");
    let deleted: Vec<_> = seeded.iter().map(|face| face.id).collect();
    let created = ["nn", "c2", "b2", "a1", "mm", "c1", "b", "a2"];
    let result = commit(&world, &execution, &created, &deleted, plan);
    let piece = |index| born(&result, "cut_piece", index).1;
    let patch = |index| born(&result, "seam_patch", index).1;
    for (patch_index, piece_index) in [(11, 1), (13, 3), (14, 1), (15, 2), (16, 2)] {
        assert!(
            embeds(&patch(patch_index), &piece(piece_index)),
            "patch {patch_index} names piece {piece_index}"
        );
    }
    // A referenced split child and survivor carry their parents' identities.
    assert_eq!(piece(1).parts().parent_identities(), &[p.identity]);
    let mut merged = [m1.identity, m2.identity];
    merged.sort();
    assert_eq!(piece(3).parts().parent_identities(), &merged);
}
