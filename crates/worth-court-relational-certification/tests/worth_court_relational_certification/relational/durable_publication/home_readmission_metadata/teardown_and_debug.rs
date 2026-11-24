//! Opaque Debug formatting and owner teardown / recovery-replacement revocation.
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_relational::facade::{
    branch::RelationalBranchBasisDenial,
    durability::RecoveryVerificationMode,
    identity::PartitionId,
    runtime::{HomeReadmissionMetadataReadDenial, RelationalRuntime},
    symbols::ClientKey,
    transactions::{CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
};
use worth_testkit_relational::{
    aspect_storage::publications::begin_main_transaction,
    durable_publication::field_patch,
    shared_relational_certification_test_execution_policy::serial_request,
};

use super::{
    runtime_with_admitted_metadata_kinds, seed_entity, APP_ENTITY_KIND, METADATA_ENTITY_KIND,
    METADATA_RELATION_KIND,
};

fn recovery_plan(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
) -> worth_relational::facade::durability::RecoveryPlan {
    runtime
        .durability()
        .recovery_plan(execution, RecoveryVerificationMode::NormalRecoveryVerification)
        .expect("recovery plan")
}

fn seed_named_app_entity(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    sentinel_value: &str,
) {
    let mut tx = begin_main_transaction(runtime);
    tx.push_batch(
        WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: APP_ENTITY_KIND,
                client_key: ClientKey::raw(key),
                fields: field_patch("name", "name", sentinel_value),
            },
        ))),
        ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("app entity fits");
    tx.commit(execution, runtime).expect("app entity commits");
}

#[test]
fn debug_formatting_of_metadata_view_and_observations_is_opaque() {
    let host_request = serial_request();
    let execution = &ExecutionRequest::serial(&host_request);

    let source = runtime_with_admitted_metadata_kinds();
    let secret_payload = "secret-application-payload-r7";
    seed_named_app_entity(execution, &source, "secret-app-key", secret_payload);
    seed_entity(execution, &source, METADATA_ENTITY_KIND, "meta-key-r7");
    source
        .durability_authority()
        .checkpoint(execution)
        .expect("checkpoint source");

    let mut recovered = runtime_with_admitted_metadata_kinds();
    let (_, authority) = recovered
        .durability_recovery()
        .recover_with_authority(execution, recovery_plan(&source, execution))
        .expect("recover runtime");
    let (_, basis) = recovered
        .observe_branch(&recovered.main_branch_identity())
        .expect("observe recovered branch");
    let observation = basis.observation();
    let meta_auth = authority
        .begin_home_readmission_metadata()
        .expect("issue metadata authority");
    let view = recovered
        .read_truth()
        .project_home_readmission_metadata(&meta_auth, &observation)
        .expect("open metadata view");

    let basis_debug = format!("{basis:?}");
    let observation_debug = format!("{observation:?}");
    let active_view_debug = format!("{view:?}");
    authority.finish_home_readmission();
    let closed_view_debug = format!("{view:?}");

    for rendered in [
        &basis_debug,
        &observation_debug,
        &active_view_debug,
        &closed_view_debug,
    ] {
        assert!(
            !rendered.contains(secret_payload),
            "Debug formatting must never traverse pinned root payloads: {rendered}"
        );
        assert!(
            !rendered.contains("secret-app-key"),
            "Debug formatting must never expose entity keys from pinned roots: {rendered}"
        );
    }
    assert!(active_view_debug.contains("active: true"));
    assert!(closed_view_debug.contains("active: false"));
}

#[test]
fn recovery_replacement_and_owner_drop_revoke_retained_metadata_views() {
    let host_request = serial_request();
    let execution = &ExecutionRequest::serial(&host_request);

    let source = runtime_with_admitted_metadata_kinds();
    seed_entity(execution, &source, METADATA_ENTITY_KIND, "meta-r7-1");
    source
        .durability_authority()
        .checkpoint(execution)
        .expect("checkpoint source");

    // 1. Replacing a runtime via a second recovery revokes an already-open view
    //    and authority bound to the replaced owner.
    let mut target = runtime_with_admitted_metadata_kinds();
    let (_, authority_1) = target
        .durability_recovery()
        .recover_with_authority(execution, recovery_plan(&source, execution))
        .expect("first recovery");
    let (_, basis_1) = target
        .observe_branch(&target.main_branch_identity())
        .expect("observe after first recovery");
    let meta_auth_1 = authority_1
        .begin_home_readmission_metadata()
        .expect("issue first metadata authority");
    let view_1 = target
        .read_truth()
        .project_home_readmission_metadata(&meta_auth_1, &basis_1.observation())
        .expect("open view on first recovery");
    assert_eq!(
        view_1
            .bounded_entities_of_kind(execution, METADATA_ENTITY_KIND, usize::MAX)
            .expect("scan before replacement")
            .expect("fits")
            .records()
            .len(),
        1
    );

    let (_, authority_2) = target
        .durability_recovery()
        .recover_with_authority(execution, recovery_plan(&source, execution))
        .expect("second recovery replaces runtime");
    assert_eq!(
        view_1
            .bounded_entities_of_kind(execution, METADATA_ENTITY_KIND, usize::MAX)
            .unwrap_err(),
        HomeReadmissionMetadataReadDenial::PhaseClosed
    );
    assert_eq!(
        view_1
            .bounded_relations_of_kind(execution, METADATA_RELATION_KIND, usize::MAX)
            .unwrap_err(),
        HomeReadmissionMetadataReadDenial::PhaseClosed
    );
    assert_eq!(
        authority_1.begin_home_readmission_metadata().unwrap_err(),
        RelationalBranchBasisDenial::OwnerUnavailable
    );
    drop(view_1);
    assert_eq!(
        target
            .read_truth()
            .project_home_readmission_metadata(&meta_auth_1, &basis_1.observation())
            .unwrap_err(),
        RelationalBranchBasisDenial::OwnerUnavailable
    );

    // Fresh recovery authority on the replacement runtime still operates cleanly.
    let (_, basis_2) = target
        .observe_branch(&target.main_branch_identity())
        .expect("observe after second recovery");
    let meta_auth_2 = authority_2
        .begin_home_readmission_metadata()
        .expect("issue second metadata authority");
    let view_2 = target
        .read_truth()
        .project_home_readmission_metadata(&meta_auth_2, &basis_2.observation())
        .expect("open view on replacement recovery");
    assert_eq!(
        view_2
            .bounded_entities_of_kind(execution, METADATA_ENTITY_KIND, usize::MAX)
            .expect("scan on replacement")
            .expect("fits")
            .records()
            .len(),
        1
    );

    // 2. Dropping the RelationalRuntime owner revokes any retained view and
    //    authority even while the view and observation remain alive.
    drop(target);
    assert_eq!(
        view_2
            .bounded_entities_of_kind(execution, METADATA_ENTITY_KIND, usize::MAX)
            .unwrap_err(),
        HomeReadmissionMetadataReadDenial::PhaseClosed
    );
    assert_eq!(
        view_2
            .bounded_relations_of_kind(execution, METADATA_RELATION_KIND, usize::MAX)
            .unwrap_err(),
        HomeReadmissionMetadataReadDenial::PhaseClosed
    );
    assert_eq!(
        authority_2.begin_home_readmission_metadata().unwrap_err(),
        RelationalBranchBasisDenial::OwnerUnavailable
    );
}
