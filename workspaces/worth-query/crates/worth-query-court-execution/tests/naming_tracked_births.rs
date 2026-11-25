//! Tracked birth reads through a real Relational world: a held name's
//! recorded birth (`entity_birth_of`) across seed, split, merge, retire,
//! restore, index rebuild and fork, and the name a birth of given parts was
//! minted as (`entity_name_for_birth`), with the awaited-birth dependency a
//! later commit stales.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::{
    NameSelection, RecordedNameBirth, ResolvedName, WorthQueryName,
};
use worth_query_execution_naming::facade::{
    ExecutionNameReadDenial, ExecutionNameReader, NameReadCurrentness, NameReadDependencies,
    WorthQueryOutputBirth,
};
use worth_relational::facade::mvcc::RelationalBranchObservation;

use crate::naming_execution_fixture::{
    execution_lease_for_workers, CadNamingSchema, Face, FaceSplitDisc,
};
use crate::naming_tracked_rebuild::recover;
use crate::naming_tracked_world::{NamedFace, Rebirth, TrackedWorld};

type Birth = Result<RecordedNameBirth<Face>, ExecutionNameReadDenial>;

fn birth_at(
    reader: &ExecutionNameReader<'_>,
    held: WorthQueryName<Face>,
    deps: &mut NameReadDependencies,
) -> Birth {
    reader.entity_birth_of::<CadNamingSchema, Face>(&held, deps)
}

fn side(birth: &RecordedNameBirth<Face>) -> Option<FaceSplitDisc> {
    birth
        .decode_discriminator::<FaceSplitDisc>()
        .expect("a recorded face discriminator decodes")
}

fn authored(occurrence: &str) -> WorthQueryOutputBirth {
    WorthQueryOutputBirth::new(occurrence, "authored")
        .with_discriminator(&FaceSplitDisc::Side { side: 0 })
}

fn piece(occurrence: &str, side: u8) -> WorthQueryOutputBirth {
    WorthQueryOutputBirth::new(occurrence, "cut_piece")
        .with_discriminator(&FaceSplitDisc::Side { side })
}

#[test]
fn a_held_name_reads_its_recorded_birth_whatever_its_lineage_did_since() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::new();
    let (seeded, _) = world.seed(&execution, "occ.seed", &["f1", "f2"]);
    let (f1, f2) = (seeded[0], seeded[1]);
    let (pieces, _) = world.split(&execution, f1, "occ.cut", &[("a", 1), ("b", 2)]);
    let (a, b) = (pieces[0], pieces[1]);
    let (merged, _) = world.merge_minting(&execution, a, b, "m");
    world.retire(&execution, f2);
    let head = world.head();
    let reader = world.reader(&execution, &head);
    let mut deps = NameReadDependencies::new();

    // A split-away parent still has its authored birth.
    let parent = birth_at(&reader, f1.name, &mut deps).expect("f1's birth");
    assert_eq!(parent.name(), f1.name);
    assert_eq!(
        (parent.occurrence_key(), parent.role_path()),
        ("occ.seed.f1", "authored")
    );
    assert!(parent.parents().is_empty());
    assert_eq!(side(&parent), Some(FaceSplitDisc::Side { side: 0 }));
    // A merged-away piece names its split parent and its side.
    let piece_a = birth_at(&reader, a.name, &mut deps).expect("a's birth");
    assert_eq!(piece_a.parents(), &[f1.name]);
    assert_eq!(
        (piece_a.occurrence_key(), piece_a.role_path()),
        ("occ.cut", "cut_piece")
    );
    assert_eq!(side(&piece_a), Some(FaceSplitDisc::Side { side: 1 }));
    // A minted merge result names both parents, sorted by digest.
    let minted = birth_at(&reader, merged.name, &mut deps).expect("m's birth");
    let mut parents = vec![a.name, b.name];
    parents.sort();
    assert_eq!(minted.parents(), parents.as_slice());
    assert_eq!(side(&minted), Some(FaceSplitDisc::Side { side: 9 }));
    // A retired name keeps its birth.
    let retired = birth_at(&reader, f2.name, &mut deps).expect("f2's birth");
    assert_eq!(retired.occurrence_key(), "occ.seed.f2");
    assert!(
        deps.is_empty(),
        "a visible birth is immutable and rests on nothing"
    );

    // Restoring the retired name re-mints the same identity: one birth.
    let (restored, _) = world.rebirth(&execution, f2, ("occ.seed", "f2", "f2r"), Rebirth::Restore);
    let head = world.head();
    let reader = world.reader(&execution, &head);
    assert_eq!(restored.name, f2.name);
    assert_eq!(birth_at(&reader, f2.name, &mut deps), Ok(retired));
}

#[test]
fn a_birth_after_the_basis_is_unknown_and_its_commit_stales_the_read() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::new();
    world.seed(&execution, "occ.seed", &["f1"]);
    let before: RelationalBranchObservation = world.head();
    let (later, born) = world.seed(&execution, "occ.later", &["f9"]);
    let f9 = later[0];

    let mut deps = NameReadDependencies::new();
    let reader = world.reader(&execution, &before);
    assert_eq!(
        birth_at(&reader, f9.name, &mut deps),
        Err(ExecutionNameReadDenial::UnknownName),
        "a birth recorded after the basis is invisible there"
    );
    let NameReadCurrentness::Stale(staleness) = deps.currentness_after_commit(&born) else {
        panic!("the commit that records the awaited birth stales the read");
    };
    assert_eq!(staleness.births, vec![*f9.name.as_bytes()]);

    let head = world.head();
    let mut fresh = NameReadDependencies::new();
    let reader = world.reader(&execution, &head);
    assert_eq!(
        birth_at(&reader, f9.name, &mut fresh).map(|birth| birth.name()),
        Ok(f9.name)
    );
}

#[test]
fn name_for_birth_answers_none_until_that_birth_is_recorded_then_its_name() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::new();
    world.seed(&execution, "occ.seed", &["f0"]);
    let lookup = |birth: &WorthQueryOutputBirth,
                  parents: &[WorthQueryName<Face>],
                  deps: &mut NameReadDependencies| {
        let head = world.head();
        world
            .reader(&execution, &head)
            .entity_name_for_birth::<CadNamingSchema, Face>(birth, parents, deps)
    };

    let mut awaiting = NameReadDependencies::new();
    assert_eq!(
        lookup(&authored("occ.seed.f1"), &[], &mut awaiting),
        Ok(None)
    );
    let (seeded, born) = world.seed(&execution, "occ.seed", &["f1"]);
    let f1 = seeded[0];
    assert!(
        matches!(
            awaiting.currentness_after_commit(&born),
            NameReadCurrentness::Stale(_)
        ),
        "recording the looked-up birth stales the `None` answer"
    );
    let mut deps = NameReadDependencies::new();
    assert_eq!(
        lookup(&authored("occ.seed.f1"), &[], &mut deps),
        Ok(Some(f1.name))
    );
    assert!(deps.is_empty());
    // Another discriminator, occurrence or parent set is another birth.
    let other_side = WorthQueryOutputBirth::new("occ.seed.f1", "authored")
        .with_discriminator(&FaceSplitDisc::Side { side: 1 });
    assert_eq!(lookup(&other_side, &[], &mut deps), Ok(None));

    // A split piece is found by its parent and side even once it is no
    // longer live, and resolving that name then reports what became of it.
    let (pieces, _) = world.split(&execution, f1, "occ.cut", &[("a", 1), ("b", 2)]);
    world.retire(&execution, pieces[0]);
    let mut deps = NameReadDependencies::new();
    let found = lookup(&piece("occ.cut", 1), &[f1.name], &mut deps);
    assert_eq!(found, Ok(Some(pieces[0].name)));
    assert_eq!(
        lookup(&piece("occ.cut", 2), &[f1.name], &mut deps),
        Ok(Some(pieces[1].name))
    );
    assert_eq!(lookup(&piece("occ.cut", 1), &[], &mut deps), Ok(None));
    let head = world.head();
    let reader = world.reader(&execution, &head);
    let resolve = |held: &WorthQueryName<Face>, deps: &mut NameReadDependencies| {
        reader.resolve_entity::<CadNamingSchema, Face>(held, &NameSelection::Exact, deps)
    };
    assert!(matches!(
        resolve(&pieces[0].name, &mut deps),
        Ok(ResolvedName::Retired { .. })
    ));
    assert!(matches!(
        resolve(&f1.name, &mut deps),
        Ok(ResolvedName::Exact(id)) if id == pieces[1].id
    ));
}

#[test]
fn a_parent_not_yet_recorded_awaits_its_own_birth() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::new();
    world.seed(&execution, "occ.seed", &["f0"]);
    let before = world.head();
    let (later, born) = world.seed(&execution, "occ.later", &["p"]);
    let mut deps = NameReadDependencies::new();
    let reader = world.reader(&execution, &before);
    let answer = reader.entity_name_for_birth::<CadNamingSchema, Face>(
        &piece("occ.cut", 1),
        &[later[0].name],
        &mut deps,
    );
    assert_eq!(answer, Ok(None));
    let NameReadCurrentness::Stale(staleness) = deps.currentness_after_commit(&born) else {
        panic!("the parent's birth stales the read");
    };
    assert_eq!(staleness.births, vec![*later[0].name.as_bytes()]);
}

#[test]
fn births_read_identically_after_the_index_rebuilds_and_stay_branch_local() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::new();
    let (seeded, _) = world.seed(&execution, "occ.seed", &["f1", "f2"]);
    let (pieces, _) = world.split(&execution, seeded[0], "occ.cut", &[("a", 1), ("b", 2)]);
    let held: Vec<NamedFace> = [seeded.clone(), pieces.clone()].concat();
    let births_on = |runtime: &worth_relational::facade::runtime::RelationalRuntime| {
        let head = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .expect("head basis")
            .observation();
        let reader = ExecutionNameReader::at_observation(runtime, &execution, &head);
        let mut deps = NameReadDependencies::new();
        held.iter()
            .map(|face| birth_at(&reader, face.name, &mut deps))
            .collect::<Vec<_>>()
    };
    let original = births_on(&world.runtime);
    assert!(original.iter().all(Result::is_ok));
    world
        .runtime
        .durability_authority()
        .checkpoint(&execution)
        .expect("checkpoint covering every birth");
    assert_eq!(births_on(&recover(&world.runtime, &execution)), original);

    // A fork taken before a split never sees the pieces' births.
    let fork = worth_relational::facade::history::BranchId("fork-births".to_owned());
    let main = worth_relational::facade::history::BranchId("main".to_owned());
    let (_, source) = world.runtime.observe_fork_source(&main).expect("forkable");
    world
        .runtime
        .fork_branch(&execution, fork.clone(), source)
        .expect("fork succeeds");
    let (later, _) = world.split(&execution, seeded[1], "occ.cut.2", &[("c", 1), ("d", 2)]);
    let fork_identity = world.runtime.branch_identity(&fork).expect("fork identity");
    let on_fork = world
        .runtime
        .admit_branch_basis(&fork_identity)
        .expect("fork basis")
        .observation();
    let reader = world.reader(&execution, &on_fork);
    let mut deps = NameReadDependencies::new();
    assert_eq!(
        birth_at(&reader, later[0].name, &mut deps),
        Err(ExecutionNameReadDenial::UnknownName)
    );
    assert_eq!(
        reader.entity_name_for_birth::<CadNamingSchema, Face>(
            &piece("occ.cut.2", 1),
            &[seeded[1].name],
            &mut deps,
        ),
        Ok(None)
    );
    assert!(birth_at(&reader, pieces[0].name, &mut deps).is_ok());
}
