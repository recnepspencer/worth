//! A selected B output carries its actual A edge into C's precommit check.
use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
use crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use im::OrdSet;

use super::SemanticSource;
use super::{field_fact, register, snapshot, AccountLabel, AccountStatus};
use crate::domain_computation::primary_graph::{
    invariant_projection::{
        ConsumedOutputEvidence, ConsumedOutputVerification, ConsumedOutputVerificationStop,
    },
    output_lineage::{
        invalidation::{
            FullVerificationReason, InvalidationEditAdmission, SettlementRegistration,
            SourceSettlementCurrentness,
        },
        ProductCoordinate, RecordedSettlementIdentity,
    },
    tests::fixture::{
        installed_authorization_world, live_scope, publish_relational_mutation, AuthorizationWorld,
        TouchAccountOperation,
    },
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationIdempotencyBinding,
    WorthQueryPrincipalResolutionMode,
};
use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};
use worth_relational::facade::identity::EntityId;

#[path = "precommit_chain/request_custody.rs"]
mod request_custody;
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

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
