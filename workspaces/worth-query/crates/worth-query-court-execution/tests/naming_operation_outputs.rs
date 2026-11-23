//! Slice 5C operation-output lowering for relations: a bound operation naming
//! program lowered against the commit basis's name index splits, merges and
//! retires held relation names through Relational's commit finalization.
//! (Installed-world decision readers cannot hold relation names, so the
//! relation postures are certified here against a real Relational runtime.)

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::{
    ApplicationNameSubjectKind, NameResolutionDenial, NameSelection, ResolvedName,
};
use worth_query_execution_naming::facade::{
    BoundOperationNaming, ExecutionNameResolveDenial, NameReadDependencies, OperationMergeKeeper,
    OperationNamedOutput, OperationNamingEntry, WorthQueryOutputBirth, WorthQueryOutputNameDenial,
};
use worth_relational::facade::lineage::{CommitNamingPlan, PendingBirthTarget};
use worth_relational::facade::transactions::{
    DeleteRelationIntent, EntityReference, MutationIntent, RelationMutationIntent,
};

use crate::naming_execution_fixture::{
    begin_staged_tx, create_seam_intent, execution_lease_for_workers, lowered_cad_catalog,
    release_commit_snapshot, seam_ref, Seam, SeamSplitDisc, SEAM_SCHEME_ID,
};
use crate::naming_tracked_world::{NamedFace, NamedSeam, TrackedWorld};

const SEAM: ApplicationNameSubjectKind = ApplicationNameSubjectKind::Relation;

fn piece(
    world: &Endpoints,
    (key, from, to): (&str, usize, usize),
    index: u8,
) -> OperationNamedOutput<PendingBirthTarget> {
    OperationNamedOutput {
        target: PendingBirthTarget::CreatedRelation(world.created(key, from, to)),
        scheme_id: SEAM_SCHEME_ID.to_owned(),
        birth: WorthQueryOutputBirth::new("seam-cut", "seam_piece")
            .with_discriminator(&SeamSplitDisc::Ordinal { index }),
    }
}

/// Faces seams run between: Relational holds one relation of a kind per
/// endpoint pair, so every seam below takes its own `(from, to)` pair.
struct Endpoints {
    world: TrackedWorld,
    faces: Vec<NamedFace>,
}

/// A created seam: `(key, from, to)` face indices.
type Created<'a> = (&'a str, usize, usize);

impl Endpoints {
    fn new(execution: &ExecutionRequest<'_, '_>) -> Self {
        let world = TrackedWorld::new();
        let (faces, _) = world.seed(execution, "ends", &["f0", "f1", "f2", "f3", "f4", "f5"]);
        Self { world, faces }
    }

    fn seam(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        key: &str,
        from: usize,
        to: usize,
    ) -> NamedSeam {
        self.world
            .seam(execution, key, self.faces[from], self.faces[to])
            .0
    }

    fn ends(&self, from: usize, to: usize) -> (EntityReference, EntityReference) {
        (
            EntityReference::Existing(self.faces[from].id),
            EntityReference::Existing(self.faces[to].id),
        )
    }

    fn created(
        &self,
        key: &str,
        from: usize,
        to: usize,
    ) -> worth_relational::facade::transactions::CreatedRelationRef {
        let (source, target) = self.ends(from, to);
        seam_ref(key, source, target)
    }

    fn lower(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        program: &BoundOperationNaming,
    ) -> Result<CommitNamingPlan, WorthQueryOutputNameDenial> {
        let head = self.world.head();
        self.world
            .runtime
            .with_persistent_names_at_observation(execution, &head, |view| {
                program.lower(&lowered_cad_catalog(), view)
            })
            .expect("name index readable")
    }

    /// Deletes `retired`, creates `created`, and commits `program` lowered.
    fn commit(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        retired: &[NamedSeam],
        created: &[Created<'_>],
        program: &BoundOperationNaming,
    ) {
        let plan = self.lower(execution, program).expect("program lowers");
        let mut intents: Vec<_> = retired
            .iter()
            .map(|seam| {
                MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
                    relation_id: seam.id,
                }))
            })
            .collect();
        intents.extend(created.iter().map(|(key, from, to)| {
            let (source, target) = self.ends(*from, *to);
            create_seam_intent(key, source, target)
        }));
        let mut tx = begin_staged_tx(&self.world.runtime, "outputs", intents);
        tx.attach_naming_plan(plan);
        let result = self
            .world
            .runtime
            .commit_branch_transaction(execution, tx)
            .expect("lowered operation names commit");
        release_commit_snapshot(&self.world.runtime, &result);
    }

    fn resolve(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        held: &NamedSeam,
        selection: NameSelection,
    ) -> Result<
        ResolvedName<worth_relational::facade::identity::RelationId>,
        ExecutionNameResolveDenial<Seam>,
    > {
        let head = self.world.head();
        self.world
            .reader(execution, &head)
            .resolve_relation::<Seam>(&held.name, &selection, &mut NameReadDependencies::new())
    }
}

fn program(entries: Vec<OperationNamingEntry<PendingBirthTarget>>) -> BoundOperationNaming {
    let mut program = BoundOperationNaming::default();
    for entry in entries {
        program.push(entry);
    }
    program
}

#[test]
fn naming_operation_outputs_split_a_held_seam_into_ambiguous_pieces() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let ends = Endpoints::new(&execution);
    let seam = ends.seam(&execution, "s", 0, 1);
    let (s1, s2) = (("s1", 0, 2), ("s2", 0, 3));
    let split = program(vec![OperationNamingEntry::Split {
        subject: SEAM,
        parent: *seam.name.as_bytes(),
        children: vec![piece(&ends, s1, 1), piece(&ends, s2, 2)],
    }]);
    ends.commit(&execution, &[seam], &[s1, s2], &split);
    match ends.resolve(&execution, &seam, NameSelection::Exact) {
        Err(ExecutionNameResolveDenial::Resolution(NameResolutionDenial::Ambiguous {
            candidates,
        })) => {
            assert_eq!(candidates.len(), 2, "both pieces, never a least member")
        }
        other => panic!("Exact over two pieces is ambiguous: {other:?}"),
    }
    match ends.resolve(&execution, &seam, NameSelection::AllDescendants) {
        Ok(ResolvedName::Plural(pieces)) => assert_eq!(pieces.len(), 2),
        other => panic!("AllDescendants returns both pieces: {other:?}"),
    }
}

#[test]
fn naming_operation_outputs_merge_held_seams_keeping_or_minting() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let ends = Endpoints::new(&execution);
    let (a, b) = (
        ends.seam(&execution, "a", 0, 1),
        ends.seam(&execution, "b", 1, 2),
    );
    let keep = program(vec![OperationNamingEntry::Merge {
        subject: SEAM,
        parents: vec![*a.name.as_bytes(), *b.name.as_bytes()],
        keeper: OperationMergeKeeper::KeepParentInPlace(*a.name.as_bytes()),
    }]);
    ends.commit(&execution, &[b], &[], &keep);
    assert_eq!(
        ends.resolve(&execution, &b, NameSelection::Exact).unwrap(),
        ResolvedName::Exact(a.id)
    );
    // `b` now resolves to `a`'s record, but no longer designates a record of
    // its own: a later posture on `b` must never act on its successor.
    let on_merged_away = program(vec![OperationNamingEntry::Retire {
        subject: SEAM,
        name: *b.name.as_bytes(),
    }]);
    assert!(matches!(
        ends.lower(&execution, &on_merged_away),
        Err(WorthQueryOutputNameDenial::HeldNameNotLive { .. })
    ));

    let (c, d) = (
        ends.seam(&execution, "c", 2, 3),
        ends.seam(&execution, "d", 3, 4),
    );
    let mint = program(vec![OperationNamingEntry::Merge {
        subject: SEAM,
        parents: vec![*c.name.as_bytes(), *d.name.as_bytes()],
        keeper: OperationMergeKeeper::MintNew(OperationNamedOutput {
            target: PendingBirthTarget::CreatedRelation(ends.created("cd", 4, 5)),
            scheme_id: SEAM_SCHEME_ID.to_owned(),
            birth: WorthQueryOutputBirth::new("seam-merge", "seam_piece")
                .with_discriminator(&SeamSplitDisc::Ordinal { index: 9 }),
        }),
    }]);
    ends.commit(&execution, &[c, d], &[("cd", 4, 5)], &mint);
    let survivor = ends.resolve(&execution, &c, NameSelection::Exact).unwrap();
    assert_eq!(
        ends.resolve(&execution, &d, NameSelection::Exact).unwrap(),
        survivor
    );
    assert!(matches!(survivor, ResolvedName::Exact(id) if id != c.id && id != d.id));
}

#[test]
fn naming_operation_outputs_retire_a_held_seam_and_refuse_it_once_ended() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let ends = Endpoints::new(&execution);
    let seam = ends.seam(&execution, "r", 0, 1);
    let retire = program(vec![OperationNamingEntry::Retire {
        subject: SEAM,
        name: *seam.name.as_bytes(),
    }]);
    ends.commit(&execution, &[seam], &[], &retire);
    assert!(matches!(
        ends.resolve(&execution, &seam, NameSelection::Exact),
        Ok(ResolvedName::Retired { .. })
    ));
    // An ended name designates no record a later posture could act on.
    assert!(matches!(
        ends.lower(&execution, &retire),
        Err(WorthQueryOutputNameDenial::HeldNameNotLive { .. })
    ));
}
