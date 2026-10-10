use crate::facade::{
    BridgeAspectChangeWideningCause, BridgeAuthoritativePatchLoweringCounters, BridgeRouteError,
    BridgeRouteErrorKind,
};
use worth_foundational::facade::AuthoritativeAspectChangeKind;
use worth_relational::facade::change_source::{
    RelationalChangeConsistencyDenial, RelationalChangeConsistencyDenialKind,
};
use worth_relational::facade::publication::PublishedAuthoritativePatchEnvelope;

use super::patch_lowering_denial::PatchLoweringDenial;
use super::RelationalBridgePublicationDenial;
use worth_execution::ExecutionRequest;

/// Count what lowering inspects and refuse an opaque change the Bridge has
/// not admitted a widening for.
///
/// Relational already proved the patch self-consistent when it minted the
/// receipt, so each record's semantic changes match its operations one for
/// one.
pub(super) fn gate_lowering_precision(
    patch: &PublishedAuthoritativePatchEnvelope,
    admitted_widening: Option<BridgeAspectChangeWideningCause>,
    execution: ExecutionRequest<'_, '_>,
) -> Result<BridgeAuthoritativePatchLoweringCounters, PatchLoweringDenial> {
    let widening_admitted =
        admitted_widening == Some(BridgeAspectChangeWideningCause::OpaquePayloadToWholeAspect);
    let mut counters = BridgeAuthoritativePatchLoweringCounters::default();
    for record in &patch.authoritative_record_patches {
        execution.consult()?;
        let changes = record.semantic_changes.len() as u64;
        counters.record_patches_inspected += 1;
        counters.authoritative_operations_inspected +=
            record.authoritative_patch.full_grammar_operation_count() as u64;
        counters.expected_operations_materialized += changes;
        counters.semantic_changes_inspected += changes;
        let contains_opaque = record
            .semantic_changes
            .iter()
            .any(|change| change.kind() == AuthoritativeAspectChangeKind::Opaque);
        if contains_opaque && !widening_admitted {
            return Err(RelationalBridgePublicationDenial::new(
                BridgeRouteError::new(
                    BridgeRouteErrorKind::UnsupportedAuthoritativePatchPrecision,
                    "opaque authoritative change has no admitted field or whole-aspect widening",
                ),
                counters,
            )
            .into());
        }
        counters.semantic_changes_matched += changes;
    }
    retain_emitted_target_counters(patch, widening_admitted, &mut counters, execution)?;
    Ok(counters)
}

/// The Bridge denial for a change Relational found inconsistent, with the
/// Relational work carried into the lowering counters one for one.
pub(super) fn consistency_denial(
    denial: &RelationalChangeConsistencyDenial,
) -> RelationalBridgePublicationDenial {
    let detail = match denial.kind() {
        RelationalChangeConsistencyDenialKind::WidenedPrecisionClaimed => {
            "Relational publication claimed widening before Bridge admission".to_owned()
        }
        RelationalChangeConsistencyDenialKind::SemanticChangeCountMismatch
        | RelationalChangeConsistencyDenialKind::UnjustifiedSemanticChange
        | RelationalChangeConsistencyDenialKind::OpaquePostureMismatch => {
            denial.detail().to_owned()
        }
    };
    let work = denial.work();
    let counters = BridgeAuthoritativePatchLoweringCounters {
        record_patches_inspected: work.records_checked(),
        authoritative_operations_inspected: work.operations_checked(),
        expected_operations_materialized: work.expected_changes_materialized(),
        semantic_changes_inspected: work.semantic_changes_examined(),
        semantic_changes_matched: work.semantic_changes_matched(),
        ..BridgeAuthoritativePatchLoweringCounters::default()
    };
    RelationalBridgePublicationDenial::new(
        BridgeRouteError::new(
            BridgeRouteErrorKind::InvalidAuthoritativePatchSemantics,
            detail,
        ),
        counters,
    )
}

fn retain_emitted_target_counters(
    patch: &PublishedAuthoritativePatchEnvelope,
    widening_admitted: bool,
    counters: &mut BridgeAuthoritativePatchLoweringCounters,
    execution: ExecutionRequest<'_, '_>,
) -> Result<(), PatchLoweringDenial> {
    use AuthoritativeAspectChangeKind as Kind;
    for record in &patch.authoritative_record_patches {
        execution.consult()?;
        for change in &record.semantic_changes {
            execution.consult()?;
            counters.semantic_changes_emission_classified += 1;
            match change.kind() {
                Kind::FieldSet | Kind::FieldClear => counters.field_targets_emitted += 1,
                Kind::WholeAspectSet | Kind::WholeAspectClear => {
                    counters.whole_aspect_targets_emitted += 1
                }
                Kind::RelationSourceEndpoint | Kind::RelationTargetEndpoint => {
                    counters.endpoint_targets_emitted += 1
                }
                Kind::LifecycleCreate | Kind::LifecycleDelete | Kind::LifecycleRetainForAudit => {
                    counters.lifecycle_targets_emitted += 1
                }
                Kind::Opaque => {
                    counters.opaque_changes_emitted += 1;
                    if widening_admitted {
                        counters.declared_widenings += 1;
                        counters.whole_aspect_targets_emitted += 1;
                    }
                }
                Kind::StructuralCreate
                | Kind::StructuralUpdate
                | Kind::StructuralDelete
                | Kind::StructuralRetainForAudit
                | Kind::StructuralMaterializationSuspended
                | Kind::StructuralRematerialized => {}
            }
        }
    }
    Ok(())
}
