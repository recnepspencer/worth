//! A selected B output carries its actual A edge into C's precommit check.
pub use worth_query_execution::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
pub use worth_query_execution::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts;

pub use std::collections::BTreeMap;
pub use std::sync::Arc;
pub use std::time::Duration;

pub use im::OrdSet;

pub use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::field_fact;
pub use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::register;
pub use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::snapshot;
pub use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::AccountLabel;
pub use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::AccountStatus;
pub use worth_query_execution::domain_computation::primary_graph::output_lineage::semantic_source::SemanticSource;
pub use worth_query_execution::domain_computation::primary_graph::invariant_projection::consumed_output::ConsumedOutputEvidence;
pub use worth_query_execution::domain_computation::primary_graph::invariant_projection::consumed_output::verification::ConsumedOutputVerification;
pub use worth_query_execution::domain_computation::primary_graph::invariant_projection::consumed_output::verification::ConsumedOutputVerificationStop;
pub use worth_query_execution::domain_computation::primary_graph::output_lineage::invalidation::mark_state::FullVerificationReason;
pub use worth_query_execution::domain_computation::primary_graph::output_lineage::invalidation::settlement::SettlementRegistration;
pub use worth_query_execution::domain_computation::primary_graph::output_lineage::invalidation::derived::SourceSettlementCurrentness;
pub use worth_query_execution::domain_computation::primary_graph::output_lineage::ProductCoordinate;
pub use worth_query_execution::domain_computation::primary_graph::output_lineage::settlement_identity::RecordedSettlementIdentity;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
pub use worth_foundational::facade::AspectFieldLocator;
pub use worth_foundational::facade::AspectValue;
pub use worth_foundational::facade::InternedString;
pub use worth_relational::facade::identity::EntityId;

pub use worth_relational::facade::transactions::AspectFieldPatch;
pub use worth_relational::facade::transactions::EntityMutationIntent;
pub use worth_relational::facade::transactions::MutationIntent;
pub use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
pub use worth_relational::facade::transactions::WorkerIntentBatch;

pub fn publish_field(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    entity: EntityId,
    locator: AspectFieldLocator,
    value: &str,
) {
    let batch =
        WorkerIntentBatch::new("native-world-current-output-journey").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: AspectFieldPatch::from(BTreeMap::from([(
                    locator,
                    AspectValue::String(InternedString::Raw(value.to_owned())),
                )])),
            }),
        ));
    publish_relational_mutation(execution, world, batch);
}

#[path = "provider_precommit_refuses_earlier_three_hop_evidence_at_current/mod.rs"]
pub mod provider_precommit_refuses_earlier_three_hop_evidence_at_current;
