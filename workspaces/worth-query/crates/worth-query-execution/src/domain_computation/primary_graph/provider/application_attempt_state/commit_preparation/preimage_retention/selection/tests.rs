//! Owner-local evidence for exact retained-preimage selection.

use std::collections::BTreeMap;

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, CanonicalFieldPath, FieldKey, InternedString,
};
use worth_query_installation::facade::{
    InstalledCorrectionMechanism, InstalledPreImageDemand, WorthQueryOperationGraphReadScope,
};
use worth_relational::facade::identity::{EntityId, KindId, PartitionId};
use worth_relational::facade::mvcc::ValidatedMutationFootprint;
use worth_relational::facade::transactions::{
    planned_aspect_field_locator, AspectFieldPatch, EntityMutationIntent, MutationIntent,
    RecordRef, UpdateEntityFieldsIntent, WorkerIntentBatch,
};

use super::{retain_from_attempt, retain_matching, WorthQueryObservedPreImageCandidate};
use crate::domain_computation::application_aftermath::WorthQueryPreImageRetentionDenial;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;
use crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphApplicationDecisionFact;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope, AccountStatus, AuthorizationWorld,
    ExactStatusRetentionOperation,
};
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;

fn demand(field: &str) -> InstalledPreImageDemand {
    let aftermath = match field {
        "frozen" => crate::domain_computation::application_aftermath::aftermath_schema_fixture::freeze_account(),
        "note" => crate::domain_computation::application_aftermath::aftermath_schema_fixture::freeze_note(),
        other => panic!("unexpected pre-image field {other}"),
    };
    let Some(InstalledCorrectionMechanism::RecordedInverse(inverse)) = aftermath.mechanism() else {
        panic!("expected inverse");
    };
    inverse.preimage_demand().clone()
}

fn observed(field: &str, value: AspectValue, slot: u64) -> WorthQueryObservedPreImageCandidate {
    WorthQueryObservedPreImageCandidate::from_observed_field(
        crate::domain_computation::application_aftermath::aftermath_schema_fixture::freeze_account_fields_read_scope(field),
        locator("IdentityAspect", field),
        value,
        entity(slot),
        KindId(7),
    )
    .expect("fixture observation has one exact installed field")
}

fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::main(), slot, 1)
}

fn mutating(
    field: &str,
    slots: impl IntoIterator<Item = u64>,
) -> Vec<(RecordRef, AspectFieldLocator)> {
    slots
        .into_iter()
        .map(|slot| {
            (
                RecordRef::Entity(entity(slot)),
                locator("IdentityAspect", field),
            )
        })
        .collect()
}

fn locator(aspect: &str, field: &str) -> AspectFieldLocator {
    planned_aspect_field_locator(
        AspectKey::new(aspect).unwrap(),
        CanonicalFieldPath::single(FieldKey::new(field).unwrap()),
    )
}

fn retain_from_exact_mutations(
    demand: &InstalledPreImageDemand,
    candidates: &[WorthQueryObservedPreImageCandidate],
    mutation: &[(RecordRef, AspectFieldLocator)],
) -> Result<
    crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage,
    WorthQueryPreImageRetentionDenial,
> {
    retain_matching(demand, candidates, mutation.is_empty(), |candidate| {
        mutation.iter().any(|(record, locator)| {
            record == candidate.target_record() && locator == candidate.locator()
        })
    })
}

#[test]
fn real_decision_fact_and_owner_footprint_reach_selection() {
    let (demand, fact, footprint, entity) = owner_path_fixture();
    let retained = retain_from_attempt(&demand, std::iter::once(&fact), &footprint)
        .expect("real owner inputs retain exact prior truth")
        .into_parts()
        .0;
    assert_eq!(
        retained
            .field(demand.loci().first().unwrap())
            .unwrap()
            .value(),
        &AspectValue::String(InternedString::from("open"))
    );
    assert_eq!(retained.target_record(), Some(&RecordRef::Entity(entity)));
}

fn owner_path_fixture() -> (
    InstalledPreImageDemand,
    WorthQueryPrimaryGraphApplicationDecisionFact,
    ValidatedMutationFootprint,
    EntityId,
) {
    let world = installed_authorization_world(true);
    let account = world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let field = AccountStatus::reference();
    let graph = world.application.primary_provider.graph.clone();
    let locator = graph
        .layout
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ExactStatusRetentionOperation::reference())
        .unwrap();
    let read_scope = operation
        .contracts()
        .graph_reads()
        .roles()
        .iter()
        .flat_map(|role| role.read_scopes())
        .find(|scope| {
            let WorthQueryOperationGraphReadScope::NativeProjection(scope) = scope else {
                return false;
            };
            scope.entity().semantic_key() == field.entity()
                && scope.aspect().as_str() == field.aspect()
                && scope
                    .projection()
                    .mask()
                    .paths()
                    .contains(locator.field_path())
        })
        .cloned()
        .expect("installed operation carries the exact field read scope");
    let fact = WorthQueryPrimaryGraphApplicationDecisionFact::application(
        read_scope,
        WorthQueryApplicationObservedFact::Field {
            entity_id: account.entity_id(),
            kind: account.entity_kind(),
            locator: locator.clone(),
            value: AspectValue::String(InternedString::from("open")),
        },
    );
    let footprint = validated_status_footprint(&world, account.entity_id(), locator);
    let Some(InstalledCorrectionMechanism::RecordedInverse(inverse)) =
        operation.contracts().aftermath().unwrap().mechanism()
    else {
        panic!("fixture operation installs a recorded inverse");
    };
    (
        inverse.preimage_demand().clone(),
        fact,
        footprint,
        account.entity_id(),
    )
}

fn validated_status_footprint(
    world: &AuthorizationWorld,
    entity: EntityId,
    locator: AspectFieldLocator,
) -> ValidatedMutationFootprint {
    let graph = world.application.primary_provider.graph.clone();
    let validated = graph.with_runtime_mut(|runtime| {
        let fields = AspectFieldPatch::from(BTreeMap::from([(
            locator,
            AspectValue::String(InternedString::from("frozen")),
        )]));
        let mut transaction = {
            let transaction_validation_input = runtime
                .admit_branch_basis(&runtime.main_branch_identity())
                .expect("main branch binding");
            runtime
                .begin_branch_transaction(
                    &transaction_validation_input,
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .expect("owner-admitted transaction context")
        };
        transaction
            .push_batch(WorkerIntentBatch::new("selection-owner-path").push(
                MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                    UpdateEntityFieldsIntent {
                        entity_id: entity,
                        fields,
                    },
                )),
            ))
            .expect("test staging stays within configured resource budgets");
        transaction
            .validate(runtime)
            .expect("owner validates mutation")
    });
    validated
        .mutation_footprint(Some(&()))
        .into_projected()
        .expect("explicit demand projects owner footprint")
}

#[test]
fn exact_demand_slices_one_matching_field_and_preserves_owner_axes() {
    let demand = demand("frozen");
    let retained = retain_from_exact_mutations(
        &demand,
        &[
            observed("frozen", AspectValue::Bool(true), 1),
            observed("note", AspectValue::String("ignored".into()), 2),
        ],
        &mutating("frozen", [1]),
    )
    .expect("retain exact field");
    let field = retained.field(&demand.loci()[0]).unwrap();
    assert_eq!(field.value(), &AspectValue::Bool(true));
    assert_eq!(field.entity_kind(), KindId(7));
    assert_eq!(field.locator(), &locator("IdentityAspect", "frozen"));
    assert_eq!(
        retained.target_record(),
        Some(&RecordRef::Entity(entity(1)))
    );
}

#[test]
fn wrong_field_locus_and_record_cannot_supply_prior_truth() {
    let demand = demand("frozen");
    let cases = [
        (
            observed("frozen", AspectValue::Bool(true), 1),
            mutating("note", [1]),
        ),
        (
            WorthQueryObservedPreImageCandidate::from_observed_field(
                crate::domain_computation::application_aftermath::aftermath_schema_fixture::freeze_account_fields_read_scope("note"),
                locator("IdentityAspect", "note"),
                AspectValue::String("foreign".into()),
                entity(1),
                KindId(7),
            )
            .unwrap(),
            mutating("note", [1]),
        ),
        (
            observed("frozen", AspectValue::Bool(true), 1),
            mutating("frozen", [2]),
        ),
    ];
    for (candidate, mutation) in cases {
        assert_eq!(
            retain_from_exact_mutations(&demand, &[candidate], &mutation).unwrap_err(),
            WorthQueryPreImageRetentionDenial::MissingDemandedField
        );
    }
}

#[test]
fn byte_bound_and_missing_demand_fail_closed() {
    let missing = demand("frozen");
    assert_eq!(
        retain_from_exact_mutations(&missing, &[], &mutating("frozen", [1])).unwrap_err(),
        WorthQueryPreImageRetentionDenial::MissingDemandedField
    );
    let bounded = demand("note");
    let too_large = AspectValue::String(InternedString::Raw("too-large-for-bound".into()));
    assert_eq!(
        retain_from_exact_mutations(
            &bounded,
            &[observed("note", too_large, 1)],
            &mutating("note", [1]),
        )
        .unwrap_err(),
        WorthQueryPreImageRetentionDenial::ExceedsByteBound
    );
}

#[test]
fn mutated_record_wins_over_an_earlier_unmutated_observation() {
    let demand = demand("frozen");
    let retained = retain_from_exact_mutations(
        &demand,
        &[
            observed("frozen", AspectValue::Bool(false), 1),
            observed("frozen", AspectValue::Bool(true), 2),
        ],
        &mutating("frozen", [2]),
    )
    .expect("the mutated record supplies prior truth");
    assert_eq!(
        retained.field(&demand.loci()[0]).unwrap().value(),
        &AspectValue::Bool(true)
    );
    assert_eq!(
        retained.target_record(),
        Some(&RecordRef::Entity(entity(2)))
    );
}

#[test]
fn ambiguous_or_mixed_record_prior_truth_fails_closed() {
    let demand = demand("frozen");
    assert_eq!(
        retain_from_exact_mutations(
            &demand,
            &[
                observed("frozen", AspectValue::Bool(false), 1),
                observed("frozen", AspectValue::Bool(true), 2),
            ],
            &mutating("frozen", [1, 2]),
        )
        .unwrap_err(),
        WorthQueryPreImageRetentionDenial::AmbiguousDemandedField
    );

    let aftermath = crate::domain_computation::application_aftermath::aftermath_schema_fixture::freeze_account_fields();
    let Some(InstalledCorrectionMechanism::RecordedInverse(inverse)) = aftermath.mechanism() else {
        panic!("expected multi-field inverse");
    };
    let mutation = [
        (
            RecordRef::Entity(entity(1)),
            locator("IdentityAspect", "frozen"),
        ),
        (
            RecordRef::Entity(entity(2)),
            locator("IdentityAspect", "note"),
        ),
    ];
    assert_eq!(
        retain_from_exact_mutations(
            inverse.preimage_demand(),
            &[
                observed("frozen", AspectValue::Bool(true), 1),
                observed("note", AspectValue::String("foreign".into()), 2),
            ],
            &mutation,
        )
        .unwrap_err(),
        WorthQueryPreImageRetentionDenial::AmbiguousDemandedField
    );
}

#[test]
fn no_existing_mutated_record_has_no_prior_truth() {
    let demand = demand("frozen");
    assert_eq!(
        retain_from_exact_mutations(
            &demand,
            &[observed("frozen", AspectValue::Bool(true), 1)],
            &[],
        )
        .unwrap_err(),
        WorthQueryPreImageRetentionDenial::NoMutatedRecord
    );
}
