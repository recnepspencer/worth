use crate::merge::data::{
    AdoptSourceRecordPlan, BoundExecutableMergeRecordPlan, ConvergeDeletedOnBothSidesRecordPlan,
    LoweredRecordDecision, MergeExecutableClass, MergeExecutableRecordProvenance,
    MergeExecutionCompilationError, PreserveSharedRecordPlan, ReconcileRecordPlan,
    ReconciledIdentityBasis,
};
use crate::schema::data::AspectContractPlanCatalog;

use super::aspect_plan_compilation::compile_executable_aspect_plans;
use super::lineage_continuity::derive_deleted_on_both_sides_lineage_continuity;
use super::plan_compilation::SourceRecordsByRef;

pub(super) fn compile_record_plan(
    aspect_plans: &AspectContractPlanCatalog,
    source_records_by_ref: &SourceRecordsByRef<'_>,
    lowered_record: &crate::merge::data::LoweredMergePlanRecord,
) -> Result<BoundExecutableMergeRecordPlan, MergeExecutionCompilationError> {
    let source_record = source_records_by_ref
        .get(&lowered_record.record)
        .copied()
        .ok_or_else(|| MergeExecutionCompilationError::MissingSourceRecord {
            record: lowered_record.record.clone(),
        })?;
    let provenance = executable_record_provenance(lowered_record)?;
    let aspect_plan = compile_executable_aspect_plans(aspect_plans, source_record, lowered_record)?;

    match &lowered_record.record_decision {
        LoweredRecordDecision::Execute(bundle) => {
            let executable_class = provenance.executable_class;
            if !execution_bundle_matches_executable_class(executable_class, bundle.kind) {
                return Err(
                    MergeExecutionCompilationError::ExecutableClassDecisionMismatch {
                        record: lowered_record.record.clone(),
                        executable_class,
                        decision: crate::merge::data::LoweredRecordDecisionKind::Execute,
                    },
                );
            }
            compile_executable_record_plan(
                executable_class,
                source_record,
                lowered_record,
                provenance,
                aspect_plan,
            )
        }
        LoweredRecordDecision::Block(_) => {
            Err(MergeExecutionCompilationError::UnsupportedRecordDecision {
                record: lowered_record.record.clone(),
                decision: crate::merge::data::LoweredRecordDecisionKind::Block,
            })
        }
        LoweredRecordDecision::Reject(_) => {
            Err(MergeExecutionCompilationError::UnsupportedRecordDecision {
                record: lowered_record.record.clone(),
                decision: crate::merge::data::LoweredRecordDecisionKind::Reject,
            })
        }
    }
}

fn executable_record_provenance(
    lowered_record: &crate::merge::data::LoweredMergePlanRecord,
) -> Result<MergeExecutableRecordProvenance, MergeExecutionCompilationError> {
    Ok(MergeExecutableRecordProvenance {
        classification: lowered_record.classification,
        resolution_class: lowered_record.resolution_class,
        executable_class: lowered_record.executable_class.ok_or_else(|| {
            MergeExecutionCompilationError::MissingExecutableClass {
                record: lowered_record.record.clone(),
                resolution_class: lowered_record.resolution_class,
            }
        })?,
        causal_disposition: lowered_record.causal_disposition,
        policy_proof_boundary: lowered_record.policy_proof_boundary,
        applied_policies: lowered_record.applied_policies.clone(),
    })
}

fn execution_bundle_matches_executable_class(
    executable_class: MergeExecutableClass,
    bundle_kind: crate::merge::data::LoweredRecordExecutionIntentKind,
) -> bool {
    matches!(
        (executable_class, bundle_kind),
        (
            MergeExecutableClass::AdoptSourceRecord,
            crate::merge::data::LoweredRecordExecutionIntentKind::AdoptSourceRecord
        ) | (
            MergeExecutableClass::PreserveSharedRecord,
            crate::merge::data::LoweredRecordExecutionIntentKind::PreserveSharedRecord
        ) | (
            MergeExecutableClass::ReconcileRecord,
            crate::merge::data::LoweredRecordExecutionIntentKind::ReconcileRecord
        ) | (
            MergeExecutableClass::ConvergeDeletedOnBothSides,
            crate::merge::data::LoweredRecordExecutionIntentKind::ConvergeDeletedOnBothSides
        )
    )
}

fn compile_executable_record_plan(
    executable_class: MergeExecutableClass,
    source_record: &crate::merge::data::VisibleMergeRecord,
    lowered_record: &crate::merge::data::LoweredMergePlanRecord,
    provenance: MergeExecutableRecordProvenance,
    aspect_plan: std::sync::Arc<[crate::merge::data::ExecutableAspectPlan]>,
) -> Result<BoundExecutableMergeRecordPlan, MergeExecutionCompilationError> {
    match executable_class {
        MergeExecutableClass::AdoptSourceRecord => {
            let source_visible_snapshot = source_visible_snapshot(source_record, lowered_record)?;
            Ok(BoundExecutableMergeRecordPlan::AdoptSource(
                AdoptSourceRecordPlan {
                    source_record: lowered_record.record.clone(),
                    record_kind: source_record.record_kind.clone(),
                    source_visible_snapshot,
                    provenance,
                    aspect_plan,
                },
            ))
        }
        MergeExecutableClass::PreserveSharedRecord => {
            Ok(BoundExecutableMergeRecordPlan::PreserveShared(
                PreserveSharedRecordPlan {
                    record: lowered_record.record.clone(),
                    target_record: lowered_record.target_record.clone(),
                    equality_witness: crate::merge::data::SharedTruthWitness {
                        witness_digest: crate::merge::data::equality_witness_digest(
                            source_record,
                        ),
                    },
                    provenance,
                    aspect_plan,
                },
            ))
        }
        MergeExecutableClass::ReconcileRecord => compile_reconcile_record_plan(
            source_record,
            lowered_record,
            provenance,
            aspect_plan,
        ),
        MergeExecutableClass::ConvergeDeletedOnBothSides => Ok(
            BoundExecutableMergeRecordPlan::ConvergeDeletedOnBothSides(
                ConvergeDeletedOnBothSidesRecordPlan {
                    source_record: lowered_record.record.clone(),
                    target_record: lowered_record.target_record.clone(),
                    equality_witness: crate::merge::data::SharedTruthWitness {
                        witness_digest: crate::merge::data::equality_witness_digest(
                            source_record,
                        ),
                    },
                    semantics: crate::merge::data::DeletedOnBothSidesSemantics::AuthoritativeMutualDeletionConvergence,
                    lineage_continuity: derive_deleted_on_both_sides_lineage_continuity(
                        lowered_record,
                        source_record,
                    ),
                    provenance,
                },
            ),
        ),
    }
}

fn compile_reconcile_record_plan(
    source_record: &crate::merge::data::VisibleMergeRecord,
    lowered_record: &crate::merge::data::LoweredMergePlanRecord,
    provenance: MergeExecutableRecordProvenance,
    aspect_plan: std::sync::Arc<[crate::merge::data::ExecutableAspectPlan]>,
) -> Result<BoundExecutableMergeRecordPlan, MergeExecutionCompilationError> {
    if source_record.record_kind == crate::merge::data::VisibleMergeRecordKind::Relation {
        return Err(
            MergeExecutionCompilationError::PreparedAuthorityBindingMismatch {
                detail: "relation reconcile records are not executable in phase D",
            },
        );
    }
    let target_record = lowered_record.target_record.clone().ok_or_else(|| {
        MergeExecutionCompilationError::MissingTargetRecord {
            record: lowered_record.record.clone(),
        }
    })?;
    let source_visible_snapshot = source_visible_snapshot(source_record, lowered_record)?;
    Ok(BoundExecutableMergeRecordPlan::Reconcile(
        ReconcileRecordPlan {
            source_record: lowered_record.record.clone(),
            target_record: target_record.clone(),
            source_visible_snapshot,
            identity_basis: ReconciledIdentityBasis {
                source_record: lowered_record.record.clone(),
                target_record,
            },
            causal_disposition: lowered_record.causal_disposition,
            provenance,
            aspect_plan,
        },
    ))
}

fn source_visible_snapshot(
    source_record: &crate::merge::data::VisibleMergeRecord,
    lowered_record: &crate::merge::data::LoweredMergePlanRecord,
) -> Result<crate::merge::data::VisibleMergeRecordSnapshot, MergeExecutionCompilationError> {
    crate::merge::data::visible_record_snapshot(source_record).ok_or_else(|| {
        MergeExecutionCompilationError::MissingSourceSnapshot {
            record: lowered_record.record.clone(),
            record_kind: match source_record.record_kind {
                crate::merge::data::VisibleMergeRecordKind::Entity => "entity",
                crate::merge::data::VisibleMergeRecordKind::Relation => "relation",
            },
        }
    })
}
