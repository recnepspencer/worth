//! The runtime's derived name index is never authority: a runtime recovered
//! from a checkpoint and journal tail rebuilds it from canonical lineage
//! events alone and answers every `name_of` and `resolve` identically,
//! dependency footprints included.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::{NameSelection, ResolvedName};
use worth_query_execution_naming::facade::{
    ExecutionNameReadDenial, ExecutionNameResolveDenial, NameReadDependencies,
};
use worth_relational::facade::durability::RecoveryVerificationMode;
use worth_relational::facade::identity::{EntityId, RelationId};
use worth_relational::facade::runtime::RelationalRuntime;

use crate::naming_execution_fixture::{
    execution_lease_for_workers, fresh_cad_runtime, CadNamingSchema, Face, FaceSplitDisc, Seam,
};
use crate::naming_tracked_world::{NamedFace, NamedSeam, TrackedWorld};

type FaceAnswer = Result<ResolvedName<EntityId>, ExecutionNameResolveDenial<Face>>;
type SeamAnswer = Result<ResolvedName<RelationId>, ExecutionNameResolveDenial<Seam>>;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Answers {
    faces: Vec<FaceAnswer>,
    face_names: Vec<
        Result<
            worth_query_declaration::facade::application_naming::WorthQueryName<Face>,
            ExecutionNameReadDenial,
        >,
    >,
    seams: Vec<SeamAnswer>,
    pub(crate) dependencies: NameReadDependencies,
}

fn selections() -> Vec<NameSelection> {
    vec![
        NameSelection::Exact,
        NameSelection::AllDescendants,
        NameSelection::keep_side(&FaceSplitDisc::Side { side: 1 }).unwrap(),
        NameSelection::keep_side(&FaceSplitDisc::Side { side: 2 }).unwrap(),
    ]
}

/// Every answer the runtime gives at its main head for the held names.
pub(crate) fn answers(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    faces: &[NamedFace],
    seam: NamedSeam,
) -> Answers {
    answers_on(
        runtime,
        &runtime.main_branch_identity(),
        execution,
        faces,
        seam,
    )
}

/// Every answer the runtime gives at `branch`'s head for the held names.
fn answers_on(
    runtime: &RelationalRuntime,
    branch: &worth_relational::facade::branch::RelationalBranchIdentity,
    execution: &ExecutionRequest<'_, '_>,
    faces: &[NamedFace],
    seam: NamedSeam,
) -> Answers {
    let head = runtime
        .admit_branch_basis(branch)
        .expect("head basis")
        .observation();
    let reader = worth_query_execution_naming::facade::ExecutionNameReader::at_observation(
        runtime, execution, &head,
    );
    let mut dependencies = NameReadDependencies::new();
    let faces_answers = faces
        .iter()
        .flat_map(|face| {
            selections()
                .into_iter()
                .map(move |selection| (face, selection))
        })
        .map(|(face, selection)| {
            reader.resolve_entity::<CadNamingSchema, Face>(
                &face.name,
                &selection,
                &mut dependencies,
            )
        })
        .collect();
    let face_names = faces
        .iter()
        .map(|face| reader.entity_name_of::<CadNamingSchema, Face>(face.id, &mut dependencies))
        .collect();
    let seams = selections()
        .iter()
        .take(2)
        .map(|selection| reader.resolve_relation(&seam.name, selection, &mut dependencies))
        .collect();
    Answers {
        faces: faces_answers,
        face_names,
        seams,
        dependencies,
    }
}

fn recover(source: &RelationalRuntime, execution: &ExecutionRequest<'_, '_>) -> RelationalRuntime {
    let plan = source
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .expect("recovery plan builds");
    let mut reopened = fresh_cad_runtime();
    reopened
        .durability_recovery()
        .recover(execution, plan)
        .expect("checkpoint and tail recover");
    reopened
}

#[test]
fn recovered_runtime_rebuilds_the_name_index_and_answers_identically() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::new();
    let (seeded, _) = world.seed(&execution, "occ.seed", &["f1", "f2", "f3"]);
    let (f1, f2, f3) = (seeded[0], seeded[1], seeded[2]);
    let (pieces, _) = world.split(&execution, f1, "occ.cut", &[("a", 1), ("b", 2)]);
    let (a, b) = (pieces[0], pieces[1]);
    world
        .runtime
        .durability_authority()
        .checkpoint(&execution)
        .expect("checkpoint after the first split");
    let (smaller, _) = world.split(&execution, a, "occ.cut.again", &[("a1", 1), ("a2", 2)]);
    let (a1, a2) = (smaller[0], smaller[1]);
    world.merge_keeping(&execution, a1, a2);
    world.retire(&execution, f2);
    let (seam, _) = world.seam(&execution, "occ.seam", f3, b);
    let held = [f1, f2, f3, a, b, a1, a2];
    let original = answers(&world.runtime, &execution, &held, seam);
    assert!(
        original
            .faces
            .iter()
            .any(|answer| matches!(answer, Ok(ResolvedName::Plural(_)))),
        "the world exercises plural answers"
    );
    assert!(original
        .faces
        .iter()
        .any(|answer| matches!(answer, Ok(ResolvedName::Retired { .. }))));
    assert!(!original.dependencies.is_empty());

    let tail_replayed = recover(&world.runtime, &execution);
    assert_eq!(answers(&tail_replayed, &execution, &held, seam), original);

    world
        .runtime
        .durability_authority()
        .checkpoint(&execution)
        .expect("checkpoint covering every commit");
    let rebuilt = recover(&world.runtime, &execution);
    assert_eq!(answers(&rebuilt, &execution, &held, seam), original);
}

#[test]
fn recovered_fork_rebuilds_its_inherited_index_without_later_parent_history() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::new();
    let (seeded, _) = world.seed(&execution, "occ.seed", &["f1", "f2", "f3"]);
    let (f1, f2, f3) = (seeded[0], seeded[1], seeded[2]);
    let (seam, _) = world.seam(&execution, "occ.seam", f3, f2);
    let fork = worth_relational::facade::history::BranchId("fork-a".to_owned());
    let main = worth_relational::facade::history::BranchId("main".to_owned());
    let (_, source) = world
        .runtime
        .observe_fork_source(&main)
        .expect("main is forkable");
    world
        .runtime
        .fork_branch(&execution, fork.clone(), source)
        .expect("fork succeeds");
    let (pieces, _) = world.split(&execution, f1, "occ.cut", &[("a", 1), ("b", 2)]);
    world.retire(&execution, f2);
    let held = [f1, f2, f3, pieces[0], pieces[1]];
    let fork_identity = world.runtime.branch_identity(&fork).expect("fork identity");
    let on_fork = answers_on(&world.runtime, &fork_identity, &execution, &held, seam);
    assert_eq!(
        on_fork.faces[0],
        Ok(ResolvedName::Exact(f1.id)),
        "the fork does not see the parent's later split"
    );
    assert_ne!(on_fork, answers(&world.runtime, &execution, &held, seam));
    world
        .runtime
        .durability_authority()
        .checkpoint(&execution)
        .expect("checkpoint covering the fork");
    let rebuilt = recover(&world.runtime, &execution);
    let rebuilt_fork = rebuilt
        .branch_identity(&fork)
        .expect("recovered fork identity");
    assert_eq!(
        answers_on(&rebuilt, &rebuilt_fork, &execution, &held, seam),
        on_fork
    );
}
