//! A relation retired by its endpoint's deletion, never by a `Delete` intent
//! of its own, ends its live name under either cascade policy: the held name
//! reads `Retired` (never the dead record) and the same seam can be minted
//! again without a false `DuplicateBirth`.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::{NameSelection, ResolvedName};
use worth_query_execution_naming::facade::NameReadDependencies;
use worth_relational::facade::config::{CascadeDeletePolicy, RelationalRuntimeProfile};
use worth_relational::facade::identity::RelationId;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::transactions::{
    DeleteEntityIntent, EntityMutationIntent, MutationIntent,
};

use crate::naming_execution_fixture::{
    begin_staged_tx, cad_schema_registry_with_face_aspects, execution_lease_for_workers,
    release_commit_snapshot, Seam,
};
use crate::naming_tracked_world::{NamedFace, NamedSeam, TrackedWorld};

fn world(cascade: CascadeDeletePolicy) -> TrackedWorld {
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(cad_schema_registry_with_face_aspects(Vec::new(), cascade))
        .build()
        .expect("valid relational runtime");
    TrackedWorld::with_runtime(runtime)
}

/// Deletes `faces` alone: no naming plan and no relation intent.
fn delete(world: &TrackedWorld, execution: &ExecutionRequest<'_, '_>, faces: &[NamedFace]) {
    let intents = faces
        .iter()
        .map(|face| {
            MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
                entity_id: face.id,
            }))
        })
        .collect();
    let tx = begin_staged_tx(&world.runtime, "drop-endpoints", intents);
    let committed = world
        .runtime
        .commit_branch_transaction(execution, tx)
        .expect("an endpoint deletion commits");
    release_commit_snapshot(&world.runtime, &committed);
}

fn resolve(
    world: &TrackedWorld,
    execution: &ExecutionRequest<'_, '_>,
    seam: &NamedSeam,
) -> ResolvedName<RelationId> {
    let head = world.head();
    world
        .reader(execution, &head)
        .resolve_relation::<Seam>(
            &seam.name,
            &NameSelection::Exact,
            &mut NameReadDependencies::new(),
        )
        .expect("a held seam name resolves")
}

fn endpoint_deletion_retires_the_seam_name(cascade: CascadeDeletePolicy) {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = world(cascade);
    let (faces, _) = world.seed(&execution, "ends", &["f0", "f1", "f2"]);
    let seam = world.seam(&execution, "s", faces[0], faces[1]).0;
    assert_eq!(
        resolve(&world, &execution, &seam),
        ResolvedName::Exact(seam.id)
    );

    delete(&world, &execution, &faces[1..2]);
    let ended = resolve(&world, &execution, &seam);
    assert!(
        matches!(ended, ResolvedName::Retired { .. }),
        "{cascade:?}: the seam retired with its endpoint, so its name ended: {ended:?}"
    );

    // The same occurrence, role and discriminator mint the seam again.
    let again = world.remint_seam(&execution, "s", faces[0], faces[2]);
    assert_ne!(again, seam.id);
    assert_eq!(
        resolve(&world, &execution, &seam),
        ResolvedName::Exact(again)
    );
    let head = world.head();
    let carried = world
        .reader(&execution, &head)
        .relation_name_of::<Seam>(again, &mut NameReadDependencies::new())
        .expect("the re-minted seam carries its name");
    assert!(carried == seam.name, "the exact name is restored");

    // Both endpoints deleted in one commit end the name exactly once.
    delete(&world, &execution, &[faces[0], faces[2]]);
    let ended = resolve(&world, &execution, &seam);
    assert!(
        matches!(ended, ResolvedName::Retired { .. }),
        "{cascade:?}: {ended:?}"
    );
}

#[test]
fn naming_cascade_deleted_seam_retires_its_name_and_re_mints() {
    endpoint_deletion_retires_the_seam_name(CascadeDeletePolicy::CascadeDeleteRelations);
}

#[test]
fn naming_seam_retained_dangling_for_audit_retires_its_name_and_re_mints() {
    endpoint_deletion_retires_the_seam_name(CascadeDeletePolicy::RetainDanglingForAudit);
}
