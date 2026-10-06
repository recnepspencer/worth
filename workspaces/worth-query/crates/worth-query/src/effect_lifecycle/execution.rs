use super::execution_artifacts::EffectMergeAuthority;
pub use super::execution_artifacts::{
    EffectExecutionAuthority, EffectExecutionControlStopped, EffectExecutionDeferred,
    EffectExecutionDenial, EffectExecutionDenialKind, EffectExecutionSettlementDeferred,
    EffectExecutionStop, ExecutedEffectAuthorityArtifact, ExecutedEffectPlan,
};
use super::execution_bridge::execute_lowered_writeback;
use super::execution_merge::{execute_lowered_merge, MergeUniqueValueAuthority};
use super::execution_relational_scalar::execute_lowered_mutation;
use super::lowering::{LoweredEffectExecutionArtifact, LoweredEffectExecutionPlan};
use super::receipt::EffectExecutionReceipt;
use super::RelationalEffectExecutionFailure;
impl LoweredEffectExecutionPlan {
    pub(crate) fn execute_with(
        self,
        authority: EffectExecutionAuthority<'_>,
    ) -> Result<ExecutedEffectPlan, EffectExecutionStop> {
        execute_lowered_effect_plan(self, authority)
    }

    pub fn execute_receipt_with(
        self,
        mut authority: EffectExecutionAuthority<'_>,
    ) -> Result<EffectExecutionReceipt, EffectExecutionStop> {
        let executed = execute_lowered_effect_plan_with_authority(self, &mut authority)?;
        let receipt = executed.receipt();
        super::receipt_snapshot_release::release_scalar(&mut authority, &executed);
        Ok(receipt)
    }
}

pub(crate) fn execute_lowered_effect_plan(
    lowered: LoweredEffectExecutionPlan,
    mut authority: EffectExecutionAuthority<'_>,
) -> Result<ExecutedEffectPlan, EffectExecutionStop> {
    execute_lowered_effect_plan_with_authority(lowered, &mut authority)
}

pub(crate) fn execute_lowered_effect_plan_with_authority(
    lowered: LoweredEffectExecutionPlan,
    authority: &mut EffectExecutionAuthority<'_>,
) -> Result<ExecutedEffectPlan, EffectExecutionStop> {
    match lowered.artifact() {
        LoweredEffectExecutionArtifact::Mutation(declaration) => {
            if !authority.has_relational_authority() && authority.has_bridge_authority() {
                return Err(EffectExecutionStop::Denied(EffectExecutionDenial::new(
                    &lowered,
                    EffectExecutionDenialKind::AuthorityOverrideRejected,
                    "lowered relational mutation execution rejected bridge host override; the admitted lowered plan requires relational authority",
                )));
            }
            let runtime = authority.relational_runtime().ok_or_else(|| {
                EffectExecutionStop::Denied(EffectExecutionDenial::new(
                    &lowered,
                    EffectExecutionDenialKind::MissingRelationalAuthority,
                    "lowered relational mutation execution requires a relational runtime authority",
                ))
            })?;
            let commit = execute_lowered_mutation(runtime, declaration)
                .map_err(|failure| lower_relational_stop(&lowered, failure))?;
            Ok(ExecutedEffectPlan::new(
                lowered,
                ExecutedEffectAuthorityArtifact::Mutation(commit),
                1,
            ))
        }
        LoweredEffectExecutionArtifact::Merge(declaration) => {
            if !authority.has_relational_authority() && authority.has_bridge_authority() {
                return Err(EffectExecutionStop::Denied(EffectExecutionDenial::new(
                    &lowered,
                    EffectExecutionDenialKind::AuthorityOverrideRejected,
                    "lowered relational merge execution rejected bridge host override; the admitted lowered plan requires relational authority",
                )));
            }
            if !authority.has_relational_authority() {
                return Err(EffectExecutionStop::Denied(EffectExecutionDenial::new(
                    &lowered,
                    EffectExecutionDenialKind::MissingRelationalAuthority,
                    "lowered relational merge execution requires a relational runtime authority",
                )));
            }
            if authority.merge_authority() == EffectMergeAuthority::RuntimeOwnerRequired {
                return Err(EffectExecutionStop::Denied(EffectExecutionDenial::new(
                    &lowered,
                    EffectExecutionDenialKind::MergeRequiresRuntimeOwner,
                    "a lent runtime may hold an application schema; run the merge through the backend that owns it",
                )));
            }
            let runtime = authority
                .relational_runtime()
                .expect("relational authority was checked above");
            let outcome = execute_lowered_merge(
                runtime,
                declaration,
                MergeUniqueValueAuthority::NoApplicationSchema,
            )
            .map_err(|failure| lower_relational_stop(&lowered, failure))?;
            Ok(ExecutedEffectPlan::new(
                lowered,
                ExecutedEffectAuthorityArtifact::Merge(outcome),
                1,
            ))
        }
        LoweredEffectExecutionArtifact::Writeback(declaration) => {
            if !authority.has_bridge_authority() && authority.has_relational_authority() {
                return Err(EffectExecutionStop::Denied(EffectExecutionDenial::new(
                    &lowered,
                    EffectExecutionDenialKind::AuthorityOverrideRejected,
                    "lowered query writeback execution rejected relational host override; the admitted lowered plan requires bridge authority",
                )));
            }
            let runtime = authority.bridge_runtime().ok_or_else(|| {
                EffectExecutionStop::Denied(EffectExecutionDenial::new(
                    &lowered,
                    EffectExecutionDenialKind::MissingBridgeAuthority,
                    "lowered query writeback execution requires a runtime bridge authority",
                ))
            })?;
            let execution =
                execute_lowered_writeback(runtime, declaration).map_err(|(kind, message)| {
                    EffectExecutionStop::Denied(EffectExecutionDenial::new(&lowered, kind, message))
                })?;
            Ok(ExecutedEffectPlan::new(
                lowered,
                ExecutedEffectAuthorityArtifact::Writeback { execution },
                1,
            ))
        }
    }
}

fn lower_relational_stop(
    lowered: &LoweredEffectExecutionPlan,
    failure: RelationalEffectExecutionFailure,
) -> EffectExecutionStop {
    match failure {
        RelationalEffectExecutionFailure::Deferred { kind, message } => {
            EffectExecutionStop::Deferred(EffectExecutionDeferred::new(lowered, kind, message))
        }
        RelationalEffectExecutionFailure::Denied { kind, message } => {
            EffectExecutionStop::Denied(EffectExecutionDenial::new(lowered, kind, message))
        }
        RelationalEffectExecutionFailure::ControlStopped { kind, message } => {
            EffectExecutionStop::ControlStopped(super::EffectExecutionControlStopped::new(
                lowered, kind, message,
            ))
        }
        RelationalEffectExecutionFailure::SettlementDeferred(deferred) => {
            let (message, settlement) = deferred.into_parts();
            EffectExecutionStop::SettlementDeferred(EffectExecutionSettlementDeferred::new(
                lowered, message, settlement,
            ))
        }
    }
}

pub(super) fn lower_runtime_error(
    error: impl std::fmt::Debug,
    kind: EffectExecutionDenialKind,
) -> (EffectExecutionDenialKind, String) {
    (kind, format!("{error:?}"))
}
