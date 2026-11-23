//! A branch forked while its source head's settlement is in flight inherits
//! that commit's names once settlement installs its lineage: the parent's
//! derived name index gains the commit, and so does every fork taken at or
//! after it.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::{NameSelection, ResolvedName};
use worth_query_execution_naming::facade::{
    ExecutionNameBirthPlanBuilder, ExecutionNameReader, ExecutionPendingEntityBirth,
    NameReadDependencies,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::RelationalPublicationOutcome;
use worth_relational::facade::runtime::RelationalRuntime;

use crate::naming_execution_fixture::{
    begin_staged_tx, create_face_intent, execution_lease_for_workers, face_ref, fresh_cad_runtime,
    lowered_cad_catalog, release_commit_snapshot, CadNamingSchema, Face, FaceSplitDisc,
    FACE_SCHEME_ID,
};

fn head(
    runtime: &RelationalRuntime,
    branch: &BranchId,
) -> worth_relational::facade::mvcc::RelationalBranchObservation {
    let identity = runtime.branch_identity(branch).expect("branch identity");
    runtime
        .admit_branch_basis(&identity)
        .expect("head basis")
        .observation()
}

#[test]
fn a_fork_taken_mid_settlement_inherits_the_settled_commits_names() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let runtime = fresh_cad_runtime();
    let catalog = lowered_cad_catalog();
    let (main, feature) = (BranchId("main".to_owned()), BranchId("feature".to_owned()));

    let mut tx = begin_staged_tx(&runtime, "c1", vec![create_face_intent("f1")]);
    let mut plan = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    plan.add_entity_birth(
        ExecutionPendingEntityBirth::new(face_ref("f1"), FACE_SCHEME_ID, "occ.fork.f1", "authored")
            .with_discriminator(&FaceSplitDisc::Side { side: 0 }),
    )
    .expect("birth plan");
    tx.attach_naming_plan(plan.build().into_relational_plan());
    let candidate = runtime
        .prepare_branch_transaction(&execution, tx)
        .expect("named commit prepares");
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(&execution, candidate)
    else {
        panic!("the named commit publishes");
    };

    let (_, source) = runtime
        .observe_fork_source(&main)
        .expect("main is forkable");
    runtime
        .fork_branch(&execution, feature.clone(), source)
        .expect("the fork installs while settlement is in flight");

    let committed = runtime
        .settle_performed_publication(performed)
        .expect("settlement completes");
    let f1 = committed
        .created_entity(&face_ref("f1"))
        .expect("f1 created");
    release_commit_snapshot(&runtime, &committed);

    let on_main = head(&runtime, &main);
    let name = ExecutionNameReader::at_observation(&runtime, &execution, &on_main)
        .entity_name_of::<CadNamingSchema, Face>(f1, &mut NameReadDependencies::new())
        .expect("f1 is named on main");
    let on_feature = head(&runtime, &feature);
    let reader = ExecutionNameReader::at_observation(&runtime, &execution, &on_feature);
    let deps = &mut NameReadDependencies::new();
    assert_eq!(
        reader.entity_name_of::<CadNamingSchema, Face>(f1, deps),
        Ok(name),
        "the fork inherits the commit it was taken at once that commit settles"
    );
    assert_eq!(
        reader.resolve_entity::<CadNamingSchema, Face>(&name, &NameSelection::Exact, deps),
        Ok(ResolvedName::Exact(f1))
    );
}
