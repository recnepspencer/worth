mod branch_root_schema_binding;
mod branch_transition;
mod caller_association;
mod canonical_commit_artifact;
mod checkpoint_recovery;
mod derived_index_checkpoint;
mod descriptor_continuity;
mod managed_field_text;
mod merge_replay_continuity;
mod native_checkpoint;
mod rejected_and_parent_closure;
mod semantic_result;

use crate::facade::diagnostics::DiagnosticCode;
use crate::facade::durability::{
    RecoveryAuthorityContinuityMismatch, RecoveryFailureClass, RecoveryPlan,
    RecoveryVerificationMode,
};
use crate::facade::history::BranchId;
use crate::facade::identity::KindId;
use crate::facade::indexes::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use crate::facade::lineage::LineageEventKind;
use crate::facade::merge::{MergeExecutionRequest, MergeIntent};
use crate::facade::runtime::RelationalRuntimeApi;
use crate::facade::schema::DescriptorSemanticsVersion;
use crate::facade::schema::{
    HistoricalInterpretationSensitivity, ProposedSchemaTransition, SchemaDiffAtom,
    SchemaDiffDetail, SchemaElementKind, SchemaElementRef, SchemaId, SchemaPublicationImpact,
    SchemaReconciliationPolicy, SchemaStratum, SchemaSubscriberImpact, SchemaVersionId,
};
use crate::facade::transactions::TransactionCommitError;
use crate::tests::support::*;

// CONTRACT: durability
// LANES: success, failure, recovery

fn schema_transition_for_subscriber_impact(
    target_schema_version_id: SchemaVersionId,
    subscriber_impact: SchemaSubscriberImpact,
) -> ProposedSchemaTransition {
    ProposedSchemaTransition {
        source_schema_id: SchemaId("test".to_string()),
        source_schema_version_id: SchemaVersionId(target_schema_version_id.0 - 1),
        target_schema_id: SchemaId("test".to_string()),
        target_schema_version_id,
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                SchemaId("test".to_string()),
                target_schema_version_id,
                Some(KindId(1)),
                "tag",
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            subscriber_impact,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: field_key("tag"),
                required: false,
                default_expression: Some("null".into()),
            },
        )
        .with_boundary_visibility_proof(match subscriber_impact {
            SchemaSubscriberImpact::ConsumableSurfaceChanged => {
                crate::schema::data::SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable
            }
            SchemaSubscriberImpact::ContractUpgradeRequired => {
                crate::schema::data::SubscriberBoundaryVisibility::VisibleRequiresContractUptake
            }
            _ => crate::schema::data::SubscriberBoundaryVisibility::NotVisible,
        })],
    }
}
