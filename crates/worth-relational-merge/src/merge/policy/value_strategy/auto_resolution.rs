use crate::merge::data::{
    AspectComparisonState, AspectMergePolicyKind, MergeManualResolutionClass,
    MergePolicyDecisionBoundary, MergePolicyRejectClass, MergeResolvedAspectValueStrategy,
};
use worth_relational_contracts::internal_port::schema::data::aspect_contracts::lowered_plan::LoweredAspectContractBinding;

use super::value_basis::PolicyValueLookupFailure;
use super::{scalar_policy_aspect_binding, PolicyAspectValueBasis, ScalarPolicyBindingDenial};

pub(in crate::internal_port::merge::policy) enum AutoResolutionStrategy {
    NotRequired,
    Resolved(MergeResolvedAspectValueStrategy),
    RequiresManual(MergeManualResolutionClass),
    Reject(MergePolicyRejectClass),
}

pub(in crate::internal_port::merge::policy) fn resolve_aspect_value_strategy(
    record_kind: crate::merge::data::VisibleMergeRecordKind,
    binding: Option<&LoweredAspectContractBinding>,
    value_basis: Option<&PolicyAspectValueBasis>,
    comparison: AspectComparisonState,
    applied_policy: Option<&AspectMergePolicyKind>,
    decision_boundary: MergePolicyDecisionBoundary,
    causal_disposition: crate::merge::data::MergeRecordCausalDisposition,
) -> AutoResolutionStrategy {
    if decision_boundary != MergePolicyDecisionBoundary::AutoResolved {
        return AutoResolutionStrategy::NotRequired;
    }
    match applied_policy {
        Some(AspectMergePolicyKind::PreferRicher) => match comparison {
            AspectComparisonState::Equal
            | AspectComparisonState::SourceOnly
            | AspectComparisonState::Divergent => AutoResolutionStrategy::Resolved(
                MergeResolvedAspectValueStrategy::SourceVisibleValue,
            ),
            AspectComparisonState::TargetOnly => AutoResolutionStrategy::Resolved(
                MergeResolvedAspectValueStrategy::TargetVisibleValue,
            ),
            AspectComparisonState::Unavailable => AutoResolutionStrategy::NotRequired,
        },
        Some(AspectMergePolicyKind::LastWriterWins) => match comparison {
            AspectComparisonState::Equal | AspectComparisonState::SourceOnly => {
                AutoResolutionStrategy::Resolved(
                    MergeResolvedAspectValueStrategy::SourceVisibleValue,
                )
            }
            AspectComparisonState::TargetOnly => AutoResolutionStrategy::Resolved(
                MergeResolvedAspectValueStrategy::TargetVisibleValue,
            ),
            AspectComparisonState::Divergent => match causal_disposition {
                crate::merge::data::MergeRecordCausalDisposition::SourceAfterTarget
                | crate::merge::data::MergeRecordCausalDisposition::SourceOnly => {
                    AutoResolutionStrategy::Resolved(
                        MergeResolvedAspectValueStrategy::SourceVisibleValue,
                    )
                }
                crate::merge::data::MergeRecordCausalDisposition::SourceBeforeTarget
                | crate::merge::data::MergeRecordCausalDisposition::TargetOnly => {
                    AutoResolutionStrategy::Resolved(
                        MergeResolvedAspectValueStrategy::TargetVisibleValue,
                    )
                }
                crate::merge::data::MergeRecordCausalDisposition::Concurrent
                | crate::merge::data::MergeRecordCausalDisposition::Equal => {
                    AutoResolutionStrategy::Reject(
                        MergePolicyRejectClass::LastWriterWinsCausalConflict,
                    )
                }
            },
            AspectComparisonState::Unavailable => AutoResolutionStrategy::NotRequired,
        },
        Some(AspectMergePolicyKind::MonotonicCounter) => {
            match scalar_policy_aspect_binding(record_kind, binding) {
                Ok(_) => monotonic_counter_strategy(value_basis, comparison),
                Err(ScalarPolicyBindingDenial::MissingBinding)
                | Err(ScalarPolicyBindingDenial::InvalidBuiltInPolicyValueShape) => {
                    AutoResolutionStrategy::Reject(
                        MergePolicyRejectClass::InvalidBuiltInPolicyValueShape,
                    )
                }
            }
        }
        Some(AspectMergePolicyKind::AdditiveSet) => {
            AutoResolutionStrategy::Reject(MergePolicyRejectClass::InvalidBuiltInPolicyValueShape)
        }
        _ => match comparison {
            AspectComparisonState::Equal | AspectComparisonState::SourceOnly => {
                AutoResolutionStrategy::Resolved(
                    MergeResolvedAspectValueStrategy::SourceVisibleValue,
                )
            }
            AspectComparisonState::TargetOnly | AspectComparisonState::Divergent => {
                AutoResolutionStrategy::Resolved(
                    MergeResolvedAspectValueStrategy::TargetVisibleValue,
                )
            }
            AspectComparisonState::Unavailable => AutoResolutionStrategy::NotRequired,
        },
    }
}

fn monotonic_counter_strategy(
    value_basis: Option<&PolicyAspectValueBasis>,
    comparison: AspectComparisonState,
) -> AutoResolutionStrategy {
    let Some(value_basis) = value_basis else {
        return AutoResolutionStrategy::Reject(
            MergePolicyRejectClass::InvalidBuiltInPolicyValueShape,
        );
    };
    let numeric = value_basis.numeric();
    let resolved_value = match comparison {
        AspectComparisonState::Equal | AspectComparisonState::SourceOnly => {
            match numeric.source_i64() {
                Ok(value) => worth_foundational::facade::AspectValue::Int64(value),
                Err(
                    PolicyValueLookupFailure::InvalidValueShape
                    | PolicyValueLookupFailure::MissingField,
                ) => {
                    return AutoResolutionStrategy::Reject(
                        MergePolicyRejectClass::InvalidBuiltInPolicyValueShape,
                    );
                }
                Err(PolicyValueLookupFailure::MissingRecordBasis) => {
                    return AutoResolutionStrategy::RequiresManual(
                        MergeManualResolutionClass::MissingVisibleState,
                    );
                }
            }
        }
        AspectComparisonState::TargetOnly => match numeric.target_i64() {
            Ok(value) => worth_foundational::facade::AspectValue::Int64(value),
            Err(
                PolicyValueLookupFailure::InvalidValueShape
                | PolicyValueLookupFailure::MissingField,
            ) => {
                return AutoResolutionStrategy::Reject(
                    MergePolicyRejectClass::InvalidBuiltInPolicyValueShape,
                );
            }
            Err(PolicyValueLookupFailure::MissingRecordBasis) => {
                return AutoResolutionStrategy::RequiresManual(
                    MergeManualResolutionClass::MissingVisibleState,
                );
            }
        },
        AspectComparisonState::Divergent => {
            let source = match numeric.source_i64() {
                Ok(value) => value,
                Err(
                    PolicyValueLookupFailure::InvalidValueShape
                    | PolicyValueLookupFailure::MissingField,
                ) => {
                    return AutoResolutionStrategy::Reject(
                        MergePolicyRejectClass::InvalidBuiltInPolicyValueShape,
                    );
                }
                Err(PolicyValueLookupFailure::MissingRecordBasis) => {
                    return AutoResolutionStrategy::RequiresManual(
                        MergeManualResolutionClass::MissingVisibleState,
                    );
                }
            };
            let target = match numeric.target_i64() {
                Ok(value) => value,
                Err(
                    PolicyValueLookupFailure::InvalidValueShape
                    | PolicyValueLookupFailure::MissingField,
                ) => {
                    return AutoResolutionStrategy::Reject(
                        MergePolicyRejectClass::InvalidBuiltInPolicyValueShape,
                    );
                }
                Err(PolicyValueLookupFailure::MissingRecordBasis) => {
                    return AutoResolutionStrategy::RequiresManual(
                        MergeManualResolutionClass::MissingVisibleState,
                    );
                }
            };
            let base = match numeric.base_i64() {
                Ok(value) => value,
                Err(
                    PolicyValueLookupFailure::InvalidValueShape
                    | PolicyValueLookupFailure::MissingField,
                ) => {
                    return AutoResolutionStrategy::Reject(
                        MergePolicyRejectClass::InvalidBuiltInPolicyValueShape,
                    );
                }
                Err(PolicyValueLookupFailure::MissingRecordBasis) => {
                    return AutoResolutionStrategy::RequiresManual(
                        MergeManualResolutionClass::MissingAncestorValueBasis,
                    );
                }
            };
            worth_foundational::facade::AspectValue::Int64(source + target - base)
        }
        AspectComparisonState::Unavailable => return AutoResolutionStrategy::NotRequired,
    };
    AutoResolutionStrategy::Resolved(MergeResolvedAspectValueStrategy::InlineAspectValue(
        resolved_value,
    ))
}
