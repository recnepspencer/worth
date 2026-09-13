use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, EntityReference, MutationIntent, RelationSpec,
    WorkerIntentBatch,
};

use super::super::super::application_attempt::authenticated_principal;
use super::super::super::fixture::capability::CapabilityConflictingBeneficiary;
use super::super::super::fixture::{
    live_scope, AccountIdentity, ElevatedCapabilityTouchOperation, PrincipalIdentityField,
};
use super::super::capability_progression::time;
use crate::domain_computation::primary_graph::{
    WorthQueryOperationAuthorizationDenialKind, WorthQueryPrincipalResolutionMode,
};

#[test]
fn exact_conflicted_approver_cannot_open_elevation_authority() {
    let (world, request, approved) = super::approval_transition::exact_approved_world();
    world.authorization_time.script([time(100)]);
    let principal = authenticated_principal(&world, &request);
    add_approver_conflict(&world);

    let Err(denial) = super::admit(&world, &approved, &principal, &request, Some("elevation-2"))
    else {
        panic!("a conflict held by the exact approver must deny elevation admission");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ElevationApproverConflict
    );
}

#[test]
fn approver_conflict_drift_stales_admitted_elevation_authority() {
    let (world, request, approved) = super::approval_transition::exact_approved_world();
    world.authorization_time.script([time(100), time(100)]);
    let principal = authenticated_principal(&world, &request);
    let access =
        super::admit(&world, &approved, &principal, &request, Some("elevation-2")).unwrap();
    add_approver_conflict(&world);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ElevatedCapabilityTouchOperation::reference())
        .unwrap();

    let Err(denial) =
        world
            .application
            .authorize_capability_operation(access, &operation, Default::default())
    else {
        panic!("new conflict truth for the exact approver must stale retained authority");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
}

fn add_approver_conflict(world: &super::super::super::fixture::AuthorizationWorld) {
    let scope = live_scope();
    let approver = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            PrincipalIdentityField::reference(),
            2_u64,
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountIdentity::reference(),
            "account-1".to_owned(),
            &scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let graph = world.application.runtime.primary_graph().unwrap();
    let relation_kind = graph
        .layout()
        .relation(CapabilityConflictingBeneficiary::reference().name())
        .unwrap()
        .kind;
    super::super::super::fixture::publish_relational_mutation(
        world,
        WorkerIntentBatch::new("add-approver-conflict").push(MutationIntent::Create(
            CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: relation_kind,
                client_key: ClientKey::raw("elevation-approver-conflict-drift"),
                source: EntityReference::Existing(approver.entity_id()),
                target: EntityReference::Existing(account.entity_id()),
                fields: AspectFieldPatch::default(),
            }),
        )),
    );
}
