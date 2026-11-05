//! Owner-local evidence for exact retained-preimage selection.

pub use std::collections::BTreeMap;

pub use worth_foundational::facade::AspectFieldLocator;
pub use worth_foundational::facade::AspectKey;
pub use worth_foundational::facade::AspectValue;
pub use worth_foundational::facade::CanonicalFieldPath;
pub use worth_foundational::facade::FieldKey;
pub use worth_foundational::facade::InternedString;
pub use worth_query_installation::facade::InstalledCorrectionMechanism;
pub use worth_query_installation::facade::InstalledPreImageDemand;
pub use worth_query_installation::facade::WorthQueryOperationGraphReadScope;
pub use worth_relational::facade::identity::EntityId;
pub use worth_relational::facade::identity::KindId;
pub use worth_relational::facade::identity::PartitionId;
pub use worth_relational::facade::mvcc::ValidatedMutationFootprint;
pub use worth_relational::facade::transactions::planned_aspect_field_locator;
pub use worth_relational::facade::transactions::AspectFieldPatch;
pub use worth_relational::facade::transactions::EntityMutationIntent;
pub use worth_relational::facade::transactions::MutationIntent;
pub use worth_relational::facade::transactions::RecordRef;
pub use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
pub use worth_relational::facade::transactions::WorkerIntentBatch;

pub use crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::preimage_retention::selection::tests::owner_path_fixture::owner_path_fixture;

pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::preimage_retention::selection::retain_from_attempt;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::preimage_retention::selection::retain_matching;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::preimage_retention::selection::WorthQueryObservedPreImageCandidate;
pub use worth_query_execution::court_doors_for_test::domain_computation::vocabulary::application_aftermath::retained_preimage::WorthQueryPreImageRetentionDenial;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_decision_fact::WorthQueryPrimaryGraphApplicationDecisionFact;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ExactStatusRetentionOperation;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub fn demand(field: &str) -> InstalledPreImageDemand {
    let aftermath = match field {
        "frozen" => worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::application_aftermath::aftermath_schema_fixture::freeze_account(),
        "note" => worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::application_aftermath::aftermath_schema_fixture::freeze_note(),
        other => panic!("unexpected pre-image field {other}"),
    };
    let Some(InstalledCorrectionMechanism::RecordedInverse(inverse)) = aftermath.mechanism() else {
        panic!("expected inverse");
    };
    inverse.preimage_demand().clone()
}

pub fn observed(field: &str, value: AspectValue, slot: u64) -> WorthQueryObservedPreImageCandidate {
    WorthQueryObservedPreImageCandidate::from_observed_field_for_test(
        worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::application_aftermath::aftermath_schema_fixture::freeze_account_fields_read_scope(field),
        locator("IdentityAspect", field),
        value,
        entity(slot),
        KindId(7),
    )
    .expect("fixture observation has one exact installed field")
}

pub fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::main(), slot, 1)
}

pub fn mutating(
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

pub fn locator(aspect: &str, field: &str) -> AspectFieldLocator {
    planned_aspect_field_locator(
        AspectKey::new(aspect).unwrap(),
        CanonicalFieldPath::single(FieldKey::new(field).unwrap()),
    )
}

pub fn retain_from_exact_mutations(
    demand: &InstalledPreImageDemand,
    candidates: &[WorthQueryObservedPreImageCandidate],
    mutation: &[(RecordRef, AspectFieldLocator)],
) -> Result<
    worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::retained_preimage::WorthQueryRetainedPreImage,
    WorthQueryPreImageRetentionDenial,
>{
    retain_matching(demand, candidates, mutation.is_empty(), |candidate| {
        mutation.iter().any(|(record, locator)| {
            record == candidate.target_record_for_test() && locator == candidate.locator_for_test()
        })
    })
}

#[test]
fn real_decision_fact_and_owner_footprint_reach_selection() {
    {
        let (demand, fact, footprint, entity) = owner_path_fixture();
        let retained = retain_from_attempt(&demand, std::iter::once(&fact), &footprint)
            .expect("real owner inputs retain exact prior truth")
            .into_parts_for_test()
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
            WorthQueryObservedPreImageCandidate::from_observed_field_for_test(
                worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::application_aftermath::aftermath_schema_fixture::freeze_account_fields_read_scope("note"),
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

    let aftermath = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::application_aftermath::aftermath_schema_fixture::freeze_account_fields();
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

#[path = "common_record/mod.rs"]
pub mod common_record;
#[path = "owner_path_fixture/mod.rs"]
pub mod owner_path_fixture;
