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
struct Answers {
    faces: Vec<FaceAnswer>,
    face_names: Vec<
        Result<
            worth_query_declaration::facade::application_naming::WorthQueryName<Face>,
            ExecutionNameReadDenial,
        >,
    >,
    seams: Vec<SeamAnswer>,
    dependencies: NameReadDependencies,
}

fn selections() -> Vec<NameSelection> {
    vec![
        NameSelection::Exact,
        NameSelection::AllDescendants,
        NameSelection::keep_side(&FaceSplitDisc::Side { side: 1 }).unwrap(),
        NameSelection::keep_side(&FaceSplitDisc::Side { side: 2 }).unwrap(),
    ]
}

/// Every answer the runtime gives at its head for the held names.
fn answers(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    faces: &[NamedFace],
    seam: NamedSeam,
) -> Answers {
    let head = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
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
