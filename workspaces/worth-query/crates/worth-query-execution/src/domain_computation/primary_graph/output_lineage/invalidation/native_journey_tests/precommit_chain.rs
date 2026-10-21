//! A selected B output carries its actual A edge into C's precommit check.
use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use im::OrdSet;

use super::field_fact;
use super::register;
use super::snapshot;
use super::AccountLabel;
use super::AccountStatus;
use super::SemanticSource;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerification;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop;
use crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationReason;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementRegistration;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceSettlementCurrentness;
use crate::domain_computation::primary_graph::output_lineage::ProductCoordinate;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world;
use crate::domain_computation::primary_graph::tests::fixture::live_scope;
use crate::domain_computation::primary_graph::tests::fixture::publish_relational_mutation;
use crate::domain_computation::primary_graph::tests::fixture::AuthorizationWorld;
use crate::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome;
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;
use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::AspectValue;
use worth_foundational::facade::InternedString;
use worth_relational::facade::identity::EntityId;

use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::EntityMutationIntent;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

#[path = "precommit_chain/provider_precommit_refuses_earlier_three_hop_evidence_at_current.rs"]
mod provider_precommit_refuses_earlier_three_hop_evidence_at_current;

fn publish_field(
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
