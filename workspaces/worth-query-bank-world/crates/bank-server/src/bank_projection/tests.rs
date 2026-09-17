use bank_domain::model::{
    AccountId, AccountJournalRevision, AccountName, BankPrincipalId, BankSnapshotVersion,
    InstitutionId, Money,
};
use bank_domain::proposals::{
    BankIdempotencyKey, BankOperationScopeBinding, BankProposalEngine, BankSnapshot,
    BankSnapshotBuilder,
};
use bank_domain::schema::{
    AccountIdentity, ApplyOpeningFunding, BankPrincipalBinding, BankSchema, CreatePersonalAccount,
    PersonalOwner, Principal, SendMoney, SendMoneyOperation,
};
use worth_query_host::facade::application_installation::{
    in_memory_program, WorthQueryProgramApplicationRuntime,
};
use worth_query_host::facade::declaration::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationInvariantProjectionAuthority,
    WorthQueryApplicationPrincipalKey, WorthQueryApplicationRelationSeed,
    WorthQueryPrimaryGraphBootstrap,
};

use super::{project_send_money_decision, BankProjectionDenial};
use crate::application_definition::{validated_bank_application, BankApplication};
use crate::graph_bootstrap::{
    account_key, bind_bank_world_with_estate, bind_bank_world_with_revision_override, principal_key,
};

#[test]
fn bounded_send_projection_rejects_accounting_revision_drift() {
    let snapshot = funded_world();
    let source = source_account(&snapshot);
    let harness = ProjectionHarness::install(&snapshot, |graph| {
        bind_bank_world_with_revision_override(
            graph,
            &snapshot,
            &[],
            &[],
            source,
            AccountJournalRevision::from_posting_count(0),
        )
        .unwrap();
    });

    let completed = harness
        .projection
        .project_operation::<SendMoneyOperation, _>(|reader| {
            let source_entity = reader
                .resolve_entity(AccountIdentity::reference(), source)
                .unwrap();
            project_send_money_decision(reader, &source_entity, &send(source))
        })
        .unwrap();
    assert_eq!(
        completed.output().as_ref().err(),
        Some(&BankProjectionDenial::AccountingRevisionMismatch)
    );
}

#[test]
fn bounded_send_projection_carries_the_authoritative_starting_balance() {
    let snapshot = funded_world();
    let source = source_account(&snapshot);
    let expected = bank_domain::accounting::account_balance(snapshot.journal(), source).unwrap();
    let harness = ProjectionHarness::install(&snapshot, |graph| {
        bind_bank_world_with_estate(graph, &snapshot, &[], &[], None).unwrap();
    });

    let cold = harness
        .projection
        .project_operation::<SendMoneyOperation, _>(|reader| {
            let source_entity = reader
                .resolve_entity(AccountIdentity::reference(), source)
                .unwrap();
            project_send_money_decision(reader, &source_entity, &send(source))
        })
        .unwrap();
    let cold_work = cold.work();
    let projected = cold.into_output().unwrap();

    assert_eq!(projected.starting_balance(source), Some(expected));
    assert!(projected.snapshot().journal().is_empty());
    assert_eq!(cold_work.aggregate_lookups(), 2);
    assert_eq!(cold_work.aggregate_cache_hits(), 0);
    assert_eq!(cold_work.aggregate_rebuild_input_rows(), 1);

    let warm = harness
        .projection
        .project_operation::<SendMoneyOperation, _>(|reader| {
            let source_entity = reader
                .resolve_entity(AccountIdentity::reference(), source)
                .unwrap();
            project_send_money_decision(reader, &source_entity, &send(source))
        })
        .unwrap();
    assert_eq!(
        warm.output().as_ref().unwrap().starting_balance(source),
        Some(expected)
    );
    assert_eq!(warm.work().aggregate_lookups(), 2);
    assert_eq!(warm.work().aggregate_cache_hits(), 2);
    assert_eq!(warm.work().aggregate_rebuild_input_rows(), 0);

    drop(harness);
    let rebuilt = ProjectionHarness::install(&snapshot, |graph| {
        bind_bank_world_with_estate(graph, &snapshot, &[], &[], None).unwrap();
    });
    let rebuilt_projection = rebuilt
        .projection
        .project_operation::<SendMoneyOperation, _>(|reader| {
            let source_entity = reader
                .resolve_entity(AccountIdentity::reference(), source)
                .unwrap();
            project_send_money_decision(reader, &source_entity, &send(source))
        })
        .unwrap();
    assert_eq!(
        rebuilt_projection
            .output()
            .as_ref()
            .unwrap()
            .starting_balance(source),
        Some(expected)
    );
    assert_eq!(rebuilt_projection.work().aggregate_cache_hits(), 0);
    assert_eq!(rebuilt_projection.work().aggregate_rebuild_input_rows(), 1);
}

#[test]
fn bounded_send_projection_rejects_ambiguous_recipient_ownership() {
    let snapshot = funded_world();
    let source = source_account(&snapshot);
    let harness = ProjectionHarness::install(&snapshot, |graph| {
        bind_bank_world_with_estate(graph, &snapshot, &[], &[], None).unwrap();
        graph
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                PersonalOwner::reference(),
                "hostile-second-personal-owner",
                entity_key(principal_key(id(BankPrincipalId::new, 2).get())),
                entity_key(account_key(source)),
            ))
            .unwrap();
    });

    let completed = harness
        .projection
        .project_operation::<SendMoneyOperation, _>(|reader| {
            let source_entity = reader
                .resolve_entity(AccountIdentity::reference(), source)
                .unwrap();
            project_send_money_decision(reader, &source_entity, &send(source))
        })
        .unwrap();
    assert_eq!(
        completed.output().as_ref().err(),
        Some(&BankProjectionDenial::AmbiguousRelation("PersonalOwner"))
    );
}

pub(super) struct ProjectionHarness {
    _runtime: WorthQueryProgramApplicationRuntime<BankSchema, BankApplication>,
    pub(super) projection: WorthQueryApplicationInvariantProjectionAuthority<BankSchema>,
}

impl ProjectionHarness {
    pub(super) fn install(
        snapshot: &BankSnapshot,
        bind_world: impl FnOnce(&mut WorthQueryPrimaryGraphBootstrap<BankSchema>),
    ) -> Self {
        let mut projection = None;
        let runtime = in_memory_program(
            validated_bank_application().unwrap(),
            BankSchema::declaration().unwrap(),
            ((), (), ()),
            crate::identity_runtime::bank_application_limits(),
            |graph, installed_schema| {
                let binding = installed_schema
                    .principal_binding(BankPrincipalBinding::reference())
                    .unwrap();
                for principal in snapshot.principals() {
                    let identity = WorthQueryExternalPrincipalIdentity::new(
                        "https://hostile-provider.test.invalid",
                        format!("principal-{}", principal.get()),
                    )
                    .unwrap();
                    graph.bind_principal(
                        &binding,
                        WorthQueryApplicationPrincipalKey::<BankSchema, Principal>::new(
                            principal_key(principal.get()),
                        )
                        .unwrap(),
                        principal,
                        identity,
                        WorthQueryPrincipalMappingStatus::Enabled,
                    )?;
                }
                bind_world(graph);
                projection = Some(graph.retain_invariant_projection_authority());
                Ok(())
            },
        )
        .unwrap();
        let projection = projection.expect("the program initializer retains projection authority");
        Self {
            _runtime: runtime,
            projection,
        }
    }
}

fn funded_world() -> BankSnapshot {
    let empty = BankSnapshotBuilder::new(id(BankSnapshotVersion::new, 1))
        .institution(id(InstitutionId::new, 1))
        .principal(id(BankPrincipalId::new, 1))
        .principal(id(BankPrincipalId::new, 2))
        .institution_cash_account(id(AccountId::new, 100), id(InstitutionId::new, 1))
        .build()
        .unwrap();
    let source = create_personal_account(empty, 1, "hostile-source");
    let destination = create_personal_account(source, 2, "hostile-destination");
    let source_id = source_account(&destination);
    BankProposalEngine::prepare_opening_funding(
        &destination,
        binding(),
        &key("hostile-funding"),
        &ApplyOpeningFunding {
            institution: id(InstitutionId::new, 1),
            account: source_id,
            amount: Money::from_minor(10_000).unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone()
}

fn create_personal_account(
    snapshot: BankSnapshot,
    owner: u64,
    operation_key: &str,
) -> BankSnapshot {
    BankProposalEngine::prepare_create_personal_account(
        &snapshot,
        binding(),
        &key(operation_key),
        &CreatePersonalAccount {
            institution: id(InstitutionId::new, 1),
            owner: id(BankPrincipalId::new, owner),
            display_name: AccountName::new(operation_key).unwrap(),
        },
    )
    .unwrap()
    .proposed_snapshot()
    .clone()
}

fn send(source: AccountId) -> SendMoney {
    SendMoney {
        from: source,
        recipient: id(BankPrincipalId::new, 2),
        amount: Money::from_minor(1).unwrap(),
    }
}

fn source_account(snapshot: &BankSnapshot) -> AccountId {
    snapshot
        .primary_account(id(BankPrincipalId::new, 1))
        .unwrap()
}

pub(super) fn binding() -> BankOperationScopeBinding {
    BankOperationScopeBinding::new(
        1,
        bank_domain::proposals::BankOperationScopeSchemaBinding::new(1, 1, [2; 32], [3; 32]),
        "test-operation-authority",
        bank_domain::proposals::BankOperationScopeEntityBinding::new(0, 1, 1),
        bank_domain::proposals::BankOperationScopeEntityBinding::new(0, 7, 1),
    )
}

pub(super) fn key(value: &str) -> BankIdempotencyKey {
    BankIdempotencyKey::new(value).unwrap()
}

pub(super) fn id<T>(constructor: impl FnOnce(u64) -> Option<T>, value: u64) -> T {
    constructor(value).unwrap()
}

pub(super) fn entity_key<Schema, Entity>(
    value: String,
) -> WorthQueryApplicationEntityKey<Schema, Entity> {
    WorthQueryApplicationEntityKey::new(value).unwrap()
}
