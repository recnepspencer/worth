//! The decision reader's `name_of` through a real installed operation
//! projection and `complete_projected_dependencies`. Installed Query
//! runtimes bind no persistent-name scheme into Relational yet, so their
//! records carry no names: the read refuses `RecordNotNamed`, keys the
//! record's entity fact, and the projection still completes. An entity kind
//! the operation does not declare as a decision read cannot be named at all:
//! `name_of` and `resolve` require `Entity: OperationReads<Operation>`.

use std::time::Duration;

use std::collections::BTreeMap;
use worth_execution::ExecutionAllocationPolicy as AllocationPolicy;

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::{
    NameCandidate, NameResolutionDenial, NameSelection, ResolvedName, WorthQueryName,
};
use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_execution::facade::application_naming::{
    ExecutionNameBirthPlanBuilder, ExecutionNameReadDenial, ExecutionNameResolveDenial,
    ExecutionNamingCommitAuthority, ExecutionPendingEntityBirth, PersistentNameReadVeto,
    WorthQueryDecisionNameDenial,
};
use worth_relational::facade::config::RelationalRuntimeProfile;
use worth_relational::facade::lineage::{
    InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry, LineageRecordTarget,
    PersistentNameSubjectKind, RelationalPersistentNameReadDenial,
};
use worth_relational::facade::runtime::RelationalRuntimeApi;

use worth_relational::facade::identity::{EntityId, KindId, PartitionId};
use worth_relational::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, CreatedEntityRef, EntitySpec, MutationIntent,
};

use crate::naming_execution_fixture::{
    begin_staged_tx, execution_lease_for_workers, lowered_cad_catalog, release_commit_snapshot,
    FaceSplitDisc, FACE_SCHEME_ID,
};
use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::{
    schema_types::live_scope, world_installation::installed_authorization_world,
    Account, IdentityExecutionSchema, TouchAccountOperation,
};

/// Authorizes `TouchAccountOperation` on an open account, runs `$body` as its
/// decision projection over `$reader` and `$account`, then begins and
/// completes the projected read attempt, yielding its result.
macro_rules! complete_touch_account_projection {
    (|$reader:ident, $account:ident| $body:block) => {{
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request_for_test();
            let execution = custody.borrowed_request_for_test();
            let branch = || {
                world
                    .application
                    .select_product_branch_for_test(
                        &execution,
                        world.application.product_runtime().default_branch_for_test(),
                    )
                    .expect("the product branch remains admitted")
            };
            let external = world.authenticate("alice", Duration::from_secs(60), &request);
            let actor = branch()
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let account = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account(
                &custody, &world, "open", &request,
            );
            let operation = world
                .application
                .installed_schema()
                .installed_operation(TouchAccountOperation::reference())
                .unwrap();
            world
                .selected_product(&custody)
                .authorize_operation(&phase, &actor, &account, &operation, Default::default(), &request)
                .unwrap()
        })
        .expect("the fixture owns its installed host call");
    let (_, projection, _) = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |$reader, $account| $body,
                    AllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    AllocationPolicy::SystemAllocation,
                )
                .map_err(|denial| format!("{denial:?}"))
                .and_then(|reads| {
                    reads
                        .complete_projected_dependencies(&phase, AllocationPolicy::SystemAllocation)
                        .map(drop)
                        .map_err(|denial| format!("{denial:?}"))
                })
        })
        .expect("the fixture owns its installed host call")
    }};
}

#[test]
fn decision_name_of_on_an_unnamed_record_keys_the_record_and_completes() {
    let completed = complete_touch_account_projection!(|reader, account| {
        assert!(matches!(
            reader.name_of(account),
            Err(WorthQueryDecisionNameDenial::Read(
                ExecutionNameReadDenial::RecordNotNamed { .. }
            ))
        ));
    });
    completed.expect("a stable name refusal keyed on the record seals and completes");
}

#[test]
fn a_swallowed_untrackable_resolve_refuses_to_seal() {
    let held = held_account_name();
    let completed = complete_touch_account_projection!(|reader, account| {
        let _ = account;
        let resolved = reader
            .resolve(&held, &NameSelection::Exact)
            .expect("the read is admitted");
        assert!(matches!(
            resolved,
            Err(ExecutionNameResolveDenial::Resolution(
                NameResolutionDenial::UnknownName
            ))
        ));
    });
    let refusal = completed.expect_err("an unknown held name has no fact key, so nothing seals");
    assert!(refusal.contains("PersistentName"), "{refusal}");
}

/// An `Account` name minted on a separate runtime whose scheme governs the
/// installed world's `Account` kind, so the installed world never saw it born.
fn held_account_name() -> WorthQueryName<Account> {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let subject = <Account as ApplicationEntityMarkerIdentity<IdentityExecutionSchema>>::IDENTIFIER;
    let scheme = InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
        FACE_SCHEME_ID,
        PersistentNameSubjectKind::Entity,
        subject,
        ["authored", "cut_piece"],
        true,
        false,
    )
    .expect("account scheme");
    let mut schemes = InstalledPersistentNameSchemeRegistry::new();
    schemes.register(scheme).expect("scheme registers");
    let kind = KindId(50);
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: kind,
            kind_name: subject.to_owned(),
            schema_id: SchemaId("worth.tests.execution.naming.SideAccount".to_owned()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        })
        .and_then(|registry| {
            registry.install_persistent_name_schemes(
                schemes,
                BTreeMap::from([(
                    (PersistentNameSubjectKind::Entity, kind),
                    FACE_SCHEME_ID.to_owned(),
                )]),
            )
        })
        .expect("the account scheme binds its kind");
    let side = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(registry)
        .build()
        .expect("side runtime");
    let catalog = lowered_cad_catalog();
    let created = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: kind,
        client_key: ClientKey::raw("held"),
    };
    let create = MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: kind,
        client_key: ClientKey::raw("held"),
        fields: AspectFieldPatch::default(),
    }));
    let tx = begin_staged_tx(&side, "held", vec![create]);
    let mut plan = ExecutionNameBirthPlanBuilder::new(&catalog, std::iter::empty());
    plan.add_entity_birth(
        ExecutionPendingEntityBirth::new(created.clone(), FACE_SCHEME_ID, "occ.held", "authored")
            .with_discriminator(&FaceSplitDisc::Side { side: 0 }),
    )
    .expect("birth plan");
    let (committed, names) = ExecutionNamingCommitAuthority::commit_with_naming_plan(
        &side,
        &execution,
        tx,
        plan.build(),
    )
    .expect("the side birth commits")
    .into_parts();
    let (_, name) = names
        .created_entity_name::<IdentityExecutionSchema, Account>(&committed, &created)
        .expect("the side name is admitted as an Account name");
    release_commit_snapshot(&side, &committed);
    name
}

#[test]
fn every_name_read_refusal_but_a_stable_unnamed_record_vetoes_sealing() {
    use PersistentNameReadVeto::{ReadFailed, UntrackedDependency};
    let record = LineageRecordTarget::Entity(EntityId::new(PartitionId::main(), 1, 1));
    let refusals = || {
        [
            ExecutionNameReadDenial::RecordNoLongerNamed { record },
            ExecutionNameReadDenial::UnknownScheme { scheme_id: None },
            ExecutionNameReadDenial::SubjectKindMismatch {
                expected: PersistentNameSubjectKind::Entity,
                actual: PersistentNameSubjectKind::Relation,
            },
            ExecutionNameReadDenial::KindIdentityMismatch {
                scheme_id: FACE_SCHEME_ID.to_owned(),
                expected_subject: "Face".to_owned(),
                requested_subject: "Edge".to_owned(),
            },
            ExecutionNameReadDenial::EmptySelectionFilter,
            ExecutionNameReadDenial::Relational(RelationalPersistentNameReadDenial::EmptyBranch),
        ]
    };
    let held = held_account_name();
    assert_eq!(PersistentNameReadVeto::for_name_of(&Ok(held)), None);
    let unnamed =
        Err::<WorthQueryName<Account>, _>(ExecutionNameReadDenial::RecordNotNamed { record });
    assert_eq!(PersistentNameReadVeto::for_name_of(&unnamed), None);
    for refusal in refusals() {
        let named = Err::<WorthQueryName<Account>, _>(refusal.clone());
        assert_eq!(
            PersistentNameReadVeto::for_name_of(&named),
            Some(ReadFailed),
            "{refusal:?}"
        );
        let resolved =
            Err::<ResolvedName<EntityId>, _>(ExecutionNameResolveDenial::<Account>::Read(refusal));
        assert_eq!(
            PersistentNameReadVeto::for_resolve(&resolved, false),
            Some(ReadFailed)
        );
    }
    let resolution = |denial| {
        Err::<ResolvedName<EntityId>, _>(ExecutionNameResolveDenial::<Account>::Resolution(denial))
    };
    let veto = PersistentNameReadVeto::for_resolve;
    assert_eq!(
        veto(&resolution(NameResolutionDenial::BasisPruned), false),
        Some(ReadFailed)
    );
    assert_eq!(
        veto(&resolution(NameResolutionDenial::UnknownName), true),
        Some(UntrackedDependency)
    );
    assert_eq!(
        veto(
            &resolution(NameResolutionDenial::Ambiguous {
                candidates: vec![NameCandidate::new(held, Vec::new())]
            }),
            false
        ),
        None
    );
    assert_eq!(
        veto(&Ok(ResolvedName::Retired { at_commit: 3 }), true),
        Some(UntrackedDependency)
    );
    assert_eq!(
        veto(&Ok(ResolvedName::Exact(record_entity(record))), false),
        None
    );
}

fn record_entity(record: LineageRecordTarget) -> EntityId {
    match record {
        LineageRecordTarget::Entity(entity) => entity,
        LineageRecordTarget::Relation(_) => unreachable!("the court record is an entity"),
    }
}
