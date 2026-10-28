use super::counters::EffectLifecycleCounters;

impl EffectLifecycleCounters {
    pub(crate) fn normalized(
        source_path_count: usize,
        workflow_authority_target_check_count: usize,
        basis_scope_check_count: usize,
    ) -> Self {
        Self {
            raw_intent_width: 1,
            normalized_effect_family_count: 1,
            workflow_declaration_family_check_count: 1,
            workflow_authority_target_check_count,
            source_path_count,
            basis_scope_check_count,
            ..Self::default()
        }
    }

    pub(crate) fn support_lookup(effect_support_row_count: usize) -> Self {
        Self {
            support_lookup_count: 1,
            support_lookup_width: effect_support_row_count,
            effect_support_row_count,
            ..Self::default()
        }
    }

    pub(crate) fn admitted(effect_support_row_count: usize) -> Self {
        Self {
            support_lookup_count: 1,
            support_lookup_width: effect_support_row_count,
            basis_scope_check_count: 1,
            admitted_effect_count: 1,
            effect_support_row_count,
            ..Self::default()
        }
    }

    pub(crate) fn authority_scoped_plan(effect_support_row_count: usize) -> Self {
        Self {
            support_lookup_count: 1,
            support_lookup_width: effect_support_row_count,
            basis_scope_check_count: 1,
            admitted_effect_count: 1,
            authority_scoped_plan_count: 1,
            effect_support_row_count,
            ..Self::default()
        }
    }

    pub(crate) fn rebind_required(effect_support_row_count: usize) -> Self {
        Self {
            support_lookup_count: 1,
            support_lookup_width: effect_support_row_count,
            basis_scope_check_count: 1,
            rebind_required_count: 1,
            effect_support_row_count,
            ..Self::default()
        }
    }

    pub(crate) fn lowered(
        effect_support_row_count: usize,
        effect_lowering_width: usize,
        effect_executor_rediscovery_count: usize,
    ) -> Self {
        Self {
            support_lookup_count: 1,
            support_lookup_width: effect_support_row_count,
            basis_scope_check_count: 1,
            admitted_effect_count: 1,
            authority_scoped_plan_count: 1,
            lowered_effect_count: 1,
            batch_lowering_count: 0,
            effect_lowering_width,
            effect_executor_rediscovery_count,
            batch_basis_reuse_count: 0,
            authority_reopen_count: 0,
            effect_support_row_count,
            ..Self::default()
        }
    }

    pub(crate) fn lowering_denied(
        effect_support_row_count: usize,
        effect_lowering_width: usize,
    ) -> Self {
        Self {
            support_lookup_count: 1,
            support_lookup_width: effect_support_row_count,
            basis_scope_check_count: 1,
            admitted_effect_count: 1,
            authority_scoped_plan_count: 1,
            lowering_denied_count: 1,
            batch_lowering_count: 0,
            effect_lowering_width,
            effect_support_row_count,
            ..Self::default()
        }
    }

    pub(crate) fn deferred(effect_support_row_count: usize) -> Self {
        Self {
            support_lookup_count: 1,
            support_lookup_width: effect_support_row_count,
            basis_scope_check_count: 1,
            deferred_count: 1,
            effect_support_row_count,
            ..Self::default()
        }
    }

    pub(crate) fn executed(
        effect_support_row_count: usize,
        effect_lowering_width: usize,
        effect_executor_rediscovery_count: usize,
        effect_execution_width: usize,
    ) -> Self {
        Self {
            support_lookup_count: 1,
            support_lookup_width: effect_support_row_count,
            basis_scope_check_count: 1,
            admitted_effect_count: 1,
            authority_scoped_plan_count: 1,
            lowered_effect_count: 1,
            batch_lowering_count: 0,
            effect_lowering_width,
            effect_executor_rediscovery_count,
            batch_basis_reuse_count: 0,
            authority_reopen_count: 0,
            executed_effect_count: 1,
            effect_execution_width,
            effect_support_row_count,
            ..Self::default()
        }
    }

    pub(crate) fn execution_denied(
        effect_support_row_count: usize,
        effect_lowering_width: usize,
        effect_executor_rediscovery_count: usize,
    ) -> Self {
        Self {
            support_lookup_count: 1,
            support_lookup_width: effect_support_row_count,
            basis_scope_check_count: 1,
            admitted_effect_count: 1,
            authority_scoped_plan_count: 1,
            lowered_effect_count: 1,
            batch_lowering_count: 0,
            effect_lowering_width,
            effect_executor_rediscovery_count,
            batch_basis_reuse_count: 0,
            authority_reopen_count: 0,
            execution_denied_count: 1,
            effect_support_row_count,
            ..Self::default()
        }
    }

    pub(crate) fn durability_unresolved(
        effect_support_row_count: usize,
        effect_lowering_width: usize,
        effect_executor_rediscovery_count: usize,
        effect_execution_width: usize,
    ) -> Self {
        Self {
            support_lookup_count: 1,
            support_lookup_width: effect_support_row_count,
            basis_scope_check_count: 1,
            admitted_effect_count: 1,
            authority_scoped_plan_count: 1,
            lowered_effect_count: 1,
            effect_lowering_width,
            effect_executor_rediscovery_count,
            executed_effect_count: 1,
            durability_unresolved_count: 1,
            effect_execution_width,
            effect_support_row_count,
            ..Self::default()
        }
    }

    pub(crate) fn denied(effect_support_row_count: usize) -> Self {
        Self {
            support_lookup_count: 1,
            support_lookup_width: effect_support_row_count,
            basis_scope_check_count: 1,
            denied_count: 1,
            effect_support_row_count,
            ..Self::default()
        }
    }

    pub(crate) fn advisory(effect_support_row_count: usize) -> Self {
        Self {
            support_lookup_count: 1,
            support_lookup_width: effect_support_row_count,
            basis_scope_check_count: 1,
            advisory_count: 1,
            effect_support_row_count,
            ..Self::default()
        }
    }

    pub(crate) fn admitted_batch(component_count: usize) -> Self {
        Self {
            admitted_effect_count: component_count,
            batch_basis_reuse_count: 1,
            effect_support_row_count: component_count,
            ..Self::default()
        }
    }

    pub(crate) fn batch_admission_denied(component_count: usize) -> Self {
        Self {
            denied_count: 1,
            raw_intent_width: component_count,
            batch_basis_reuse_count: 1,
            effect_support_row_count: component_count,
            ..Self::default()
        }
    }

    pub(crate) fn lowered_batch(
        component_count: usize,
        effect_lowering_width: usize,
        effect_executor_rediscovery_count: usize,
    ) -> Self {
        Self {
            admitted_effect_count: component_count,
            lowered_effect_count: component_count,
            batch_lowering_count: 1,
            effect_lowering_width,
            effect_executor_rediscovery_count,
            batch_basis_reuse_count: 1,
            authority_reopen_count: 0,
            effect_support_row_count: component_count,
            ..Self::default()
        }
    }

    pub(crate) fn executed_batch(
        component_count: usize,
        effect_lowering_width: usize,
        effect_executor_rediscovery_count: usize,
        effect_execution_width: usize,
    ) -> Self {
        Self {
            admitted_effect_count: component_count,
            lowered_effect_count: component_count,
            batch_lowering_count: 1,
            executed_effect_count: component_count,
            effect_lowering_width,
            effect_executor_rediscovery_count,
            batch_basis_reuse_count: 1,
            authority_reopen_count: 0,
            effect_execution_width,
            effect_support_row_count: component_count,
            ..Self::default()
        }
    }

    pub(crate) fn intent_denial(
        source_path_count: usize,
        workflow_authority_target_check_count: usize,
        basis_scope_check_count: usize,
    ) -> Self {
        Self {
            raw_intent_width: 1,
            workflow_declaration_family_check_count: 1,
            workflow_authority_target_check_count,
            source_path_count,
            basis_scope_check_count,
            denied_count: 1,
            ..Self::default()
        }
    }
}
