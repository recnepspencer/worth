use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

macro_rules! counter_getter {
    ($name:ident, $field:ident) => {
        pub fn $name(&self) -> usize {
            self.$field
        }
    };
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EffectLifecycleCounters {
    pub(super) raw_intent_width: usize,
    pub(super) normalized_effect_family_count: usize,
    pub(super) workflow_declaration_family_check_count: usize,
    pub(super) workflow_authority_target_check_count: usize,
    pub(super) source_path_count: usize,
    pub(super) support_lookup_count: usize,
    pub(super) support_lookup_width: usize,
    pub(super) basis_scope_check_count: usize,
    pub(super) admitted_effect_count: usize,
    pub(super) authority_scoped_plan_count: usize,
    pub(super) lowered_effect_count: usize,
    pub(super) batch_lowering_count: usize,
    pub(super) lowering_denied_count: usize,
    pub(super) effect_lowering_width: usize,
    pub(super) effect_executor_rediscovery_count: usize,
    pub(super) batch_basis_reuse_count: usize,
    pub(super) authority_reopen_count: usize,
    pub(super) executed_effect_count: usize,
    pub(super) execution_denied_count: usize,
    pub(super) durability_unresolved_count: usize,
    pub(super) effect_execution_width: usize,
    pub(super) advisory_count: usize,
    pub(super) rebind_required_count: usize,
    pub(super) deferred_count: usize,
    pub(super) denied_count: usize,
    pub(super) effect_support_row_count: usize,
}

impl EffectLifecycleCounters {
    pub fn evidence_identity(&self) -> WorthQueryEvidenceIdentity {
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "effect_lifecycle_counters_v1",
            )
            .field_usize(WorthQueryEvidenceTag::new("raw"), self.raw_intent_width)
            .field_usize(
                WorthQueryEvidenceTag::new("normalized"),
                self.normalized_effect_family_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("workflow_family_checks"),
                self.workflow_declaration_family_check_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("workflow_target_checks"),
                self.workflow_authority_target_check_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("source_paths"),
                self.source_path_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("support_lookups"),
                self.support_lookup_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("support_width"),
                self.support_lookup_width,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("basis_scope_checks"),
                self.basis_scope_check_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("admitted"),
                self.admitted_effect_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("authority_scoped_plans"),
                self.authority_scoped_plan_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("lowered_effects"),
                self.lowered_effect_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_lowering"),
                self.batch_lowering_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("lowering_denied"),
                self.lowering_denied_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("effect_lowering_width"),
                self.effect_lowering_width,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("effect_executor_rediscovery"),
                self.effect_executor_rediscovery_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("batch_basis_reuse"),
                self.batch_basis_reuse_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("authority_reopen"),
                self.authority_reopen_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("executed_effects"),
                self.executed_effect_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("execution_denied"),
                self.execution_denied_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("durability_unresolved"),
                self.durability_unresolved_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("effect_execution_width"),
                self.effect_execution_width,
            )
            .field_usize(WorthQueryEvidenceTag::new("advisory"), self.advisory_count)
            .field_usize(
                WorthQueryEvidenceTag::new("rebind_required"),
                self.rebind_required_count,
            )
            .field_usize(WorthQueryEvidenceTag::new("deferred"), self.deferred_count)
            .field_usize(WorthQueryEvidenceTag::new("denied"), self.denied_count)
            .field_usize(
                WorthQueryEvidenceTag::new("effect_support_rows"),
                self.effect_support_row_count,
            )
            .seal()
    }

    pub fn counter_for_reporting(&self) -> String {
        self.evidence_identity()
            .terminal_projection_for_reporting()
            .to_string()
    }

    pub(crate) fn combine(&self, other: &Self) -> Self {
        Self {
            raw_intent_width: self.raw_intent_width + other.raw_intent_width,
            normalized_effect_family_count: self.normalized_effect_family_count
                + other.normalized_effect_family_count,
            workflow_declaration_family_check_count: self.workflow_declaration_family_check_count
                + other.workflow_declaration_family_check_count,
            workflow_authority_target_check_count: self.workflow_authority_target_check_count
                + other.workflow_authority_target_check_count,
            source_path_count: self.source_path_count + other.source_path_count,
            support_lookup_count: self.support_lookup_count + other.support_lookup_count,
            support_lookup_width: self.support_lookup_width + other.support_lookup_width,
            basis_scope_check_count: self.basis_scope_check_count + other.basis_scope_check_count,
            admitted_effect_count: self.admitted_effect_count + other.admitted_effect_count,
            authority_scoped_plan_count: self.authority_scoped_plan_count
                + other.authority_scoped_plan_count,
            lowered_effect_count: self.lowered_effect_count + other.lowered_effect_count,
            batch_lowering_count: self.batch_lowering_count + other.batch_lowering_count,
            lowering_denied_count: self.lowering_denied_count + other.lowering_denied_count,
            effect_lowering_width: self.effect_lowering_width + other.effect_lowering_width,
            effect_executor_rediscovery_count: self.effect_executor_rediscovery_count
                + other.effect_executor_rediscovery_count,
            batch_basis_reuse_count: self.batch_basis_reuse_count + other.batch_basis_reuse_count,
            authority_reopen_count: self.authority_reopen_count + other.authority_reopen_count,
            executed_effect_count: self.executed_effect_count + other.executed_effect_count,
            execution_denied_count: self.execution_denied_count + other.execution_denied_count,
            durability_unresolved_count: self.durability_unresolved_count
                + other.durability_unresolved_count,
            effect_execution_width: self.effect_execution_width + other.effect_execution_width,
            advisory_count: self.advisory_count + other.advisory_count,
            rebind_required_count: self.rebind_required_count + other.rebind_required_count,
            deferred_count: self.deferred_count + other.deferred_count,
            denied_count: self.denied_count + other.denied_count,
            effect_support_row_count: self.effect_support_row_count
                + other.effect_support_row_count,
        }
    }

    counter_getter!(raw_intent_width, raw_intent_width);
    counter_getter!(
        normalized_effect_family_count,
        normalized_effect_family_count
    );
    counter_getter!(
        workflow_declaration_family_check_count,
        workflow_declaration_family_check_count
    );
    counter_getter!(
        workflow_authority_target_check_count,
        workflow_authority_target_check_count
    );
    counter_getter!(source_path_count, source_path_count);
    counter_getter!(support_lookup_count, support_lookup_count);
    counter_getter!(support_lookup_width, support_lookup_width);
    counter_getter!(basis_scope_check_count, basis_scope_check_count);
    counter_getter!(admitted_effect_count, admitted_effect_count);
    counter_getter!(authority_scoped_plan_count, authority_scoped_plan_count);
    counter_getter!(lowered_effect_count, lowered_effect_count);
    counter_getter!(lowering_denied_count, lowering_denied_count);
    counter_getter!(batch_lowering_count, batch_lowering_count);
    counter_getter!(effect_lowering_width, effect_lowering_width);
    counter_getter!(
        effect_executor_rediscovery_count,
        effect_executor_rediscovery_count
    );
    counter_getter!(batch_basis_reuse_count, batch_basis_reuse_count);
    counter_getter!(authority_reopen_count, authority_reopen_count);
    counter_getter!(executed_effect_count, executed_effect_count);
    counter_getter!(execution_denied_count, execution_denied_count);
    counter_getter!(durability_unresolved_count, durability_unresolved_count);
    counter_getter!(effect_execution_width, effect_execution_width);
    counter_getter!(advisory_count, advisory_count);
    counter_getter!(rebind_required_count, rebind_required_count);
    counter_getter!(deferred_count, deferred_count);
    counter_getter!(denied_count, denied_count);
    counter_getter!(effect_support_row_count, effect_support_row_count);
}
