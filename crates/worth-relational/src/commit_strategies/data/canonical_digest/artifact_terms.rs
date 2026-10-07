use super::{commit_strategy_digest, StrategyDigestBytes};
use crate::commit_strategies::data::{
    CanonicalStrategyInputDigest, StrategyInputSchemaName, StrategyInputSchemaVersion,
    StrategyLoweringSummary, StrategyPreviewValidationCostSummary,
};
use crate::transactions::data::{CommitValidationSummary, ExistingRecordTarget};
use crate::validation::data::InvariantCatalog;
use worth_foundational::facade::AspectFieldLocator;

pub(crate) fn lowering_summary_digest(summary: &StrategyLoweringSummary) -> [u8; 32] {
    commit_strategy_digest("strategy-lowering-summary-v1", |bytes| {
        bytes.usize(summary.worker_batch_count());
        bytes.usize(summary.total_intent_count());
        bytes.usize(summary.touched_partition_count());
        bytes.usize(summary.cross_partition_relation_count());
        bytes.usize(summary.normalized_client_key_count());
        bytes.usize(summary.lineage_transition_count());
        bytes.usize(summary.projected_entity_record_reads());
        bytes.usize(summary.projected_relation_record_reads());
        bytes.usize(summary.projected_partition_reads());
    })
}

pub(crate) fn commit_validation_summary_digest(summary: &CommitValidationSummary) -> [u8; 32] {
    commit_strategy_digest("strategy-preview-validation-summary-v1", |bytes| {
        bytes.usize(summary.execution_count);
        bytes.usize(summary.executed_count);
        bytes.usize(summary.skipped_count);
        bytes.usize(summary.committed_observation_count);
        bytes.usize(summary.speculative_observation_count);
        bytes.usize(summary.plan_backed_execution_count);
        bytes.bool(summary.commit_boundary_seen);
        bytes.bool(summary.mutation_sensitive_seen);
        bytes.bool(summary.snapshot_publication_seen);
        bytes.bool(summary.certification_boundary_seen);
        bytes.bool(summary.harness_audit_seen);
        bytes.u32(summary.consumed_groups.mask());
        bytes.u32(summary.applicable_groups.mask());
        bytes.usize(summary.result_count);
        bytes.usize(summary.advisory_count);
        bytes.usize(summary.violation_count);
        bytes.usize(summary.custom_failure_count);
        bytes.usize(summary.custom_panic_count);
        bytes.bool(summary.blocking_violation);
        bytes.bool(summary.publication_violation);
    })
}

pub(crate) fn preview_validation_cost_digest(
    summary: &StrategyPreviewValidationCostSummary,
) -> [u8; 32] {
    commit_strategy_digest("strategy-preview-validation-cost-v1", |bytes| {
        bytes.version_id(summary.preview_version_id());
        bytes.usize(summary.merged_intent_count());
        bytes.usize(summary.touched_partition_count());
        bytes.usize(summary.bulk_entity_slots_reserved());
        bytes.usize(summary.bulk_relation_slots_reserved());
        bytes.usize(summary.post_mutation_preview_pass_count());
    })
}

pub(crate) fn runtime_invariant_catalog_digest(catalog: &InvariantCatalog) -> [u8; 32] {
    commit_strategy_digest("strategy-runtime-invariant-catalog-v1", |bytes| {
        bytes.string(&catalog.canonical_registration_digest());
    })
}

pub(crate) fn native_entity_fields_scope_digest(
    entity_id: crate::identity::data::EntityId,
    targets: &[AspectFieldLocator],
) -> [u8; 32] {
    commit_strategy_digest("strategy-native-entity-fields-scope-v2", |bytes| {
        bytes.entity_id(entity_id);
        write_sorted_scope_targets(bytes, targets);
    })
}

pub(crate) fn native_entity_replacement_scope_digest(
    entity_id: crate::identity::data::EntityId,
    replacement_client_key: &str,
    targets: &[AspectFieldLocator],
) -> [u8; 32] {
    commit_strategy_digest("strategy-native-entity-replacement-scope-v2", |bytes| {
        bytes.entity_id(entity_id);
        bytes.string(replacement_client_key);
        write_sorted_scope_targets(bytes, targets);
    })
}

pub(crate) fn serial_intent_scope_digest(
    strategy_id: crate::commit_strategies::data::CommitStrategyId,
    input_schema_name: &StrategyInputSchemaName,
    input_schema_version: StrategyInputSchemaVersion,
    input_digest: CanonicalStrategyInputDigest,
    targets: &[ExistingRecordTarget],
) -> [u8; 32] {
    commit_strategy_digest("strategy-serial-intent-scope-v1", |bytes| {
        bytes.u32(strategy_id.0);
        bytes.string(input_schema_name.as_str());
        bytes.u16(input_schema_version.0);
        bytes.digest_bytes(&input_digest.0);
        bytes.usize(targets.len());
        for target in targets {
            write_existing_record_target(bytes, *target);
        }
    })
}

fn write_sorted_scope_targets(bytes: &mut StrategyDigestBytes, targets: &[AspectFieldLocator]) {
    let mut targets = targets.to_vec();
    targets.sort();
    targets.dedup();
    bytes.usize(targets.len());
    for target in targets {
        bytes.aspect_field_locator(&target);
    }
}

fn write_existing_record_target(bytes: &mut StrategyDigestBytes, target: ExistingRecordTarget) {
    match target {
        ExistingRecordTarget::Entity(entity_id) => {
            bytes.tag(1);
            bytes.entity_id(entity_id);
        }
        ExistingRecordTarget::Relation(relation_id) => {
            bytes.tag(2);
            bytes.relation_id(relation_id);
        }
    }
}
