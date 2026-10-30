use crate::query_context::{
    DiffQueryMetadata, QueryBasisMetadata, QueryBasisResultBundle, QueryContextCounters,
    QueryDiffResultBundle,
};

pub(super) fn counter_values(counters: &QueryContextCounters) -> Vec<String> {
    vec![
        format!(
            "query_basis_bindings:{}",
            counters.query_basis_binding_count()
        ),
        format!(
            "historical_basis_lookups:{}",
            counters.historical_basis_lookup_count()
        ),
        format!(
            "comparison_basis_lookups:{}",
            counters.comparison_basis_lookup_count()
        ),
        format!(
            "materialization_path_compatibility_checks:{}",
            counters.materialization_path_compatibility_check_count()
        ),
        format!("basis_binding_width:{}", counters.basis_binding_width()),
        format!(
            "historical_lookup_width:{}",
            counters.historical_lookup_width()
        ),
        "comparison_binding_width:0".to_string(),
        format!(
            "comparison_scope_width:{}",
            counters.comparison_scope_width()
        ),
        format!("diff_input_breadth:{}", counters.diff_input_breadth()),
        format!(
            "diff_change_set_row_width:{}",
            counters.diff_change_set_row_width()
        ),
        format!("denial_width:{}", counters.denial_width()),
        format!(
            "unsupported_denials:{}",
            counters.unsupported_basis_denial_count()
        ),
        format!(
            "basis_substitution_denials:{}",
            counters.basis_substitution_denial_count()
        ),
        format!(
            "comparison_broadening_denials:{}",
            counters.comparison_broadening_denial_count()
        ),
        format!(
            "historical_broadening_denials:{}",
            counters.historical_broadening_denial_count()
        ),
        format!(
            "predicted_comparison_width:{}",
            counters.comparison_row_width()
        ),
        "realized_comparison_width:0".to_string(),
        "metadata_attachment_width:0".to_string(),
        "query_context_execution_count:0".to_string(),
        "query_context_metadata_attachment_count:0".to_string(),
        "query_context_executor_rediscovery:0".to_string(),
        format!("basis_rediscovery:{}", counters.basis_rediscovery_count()),
        format!(
            "historical_path_rediscovery:{}",
            counters.historical_path_rediscovery_count()
        ),
        format!(
            "comparison_family_rediscovery:{}",
            counters.comparison_family_rediscovery_count()
        ),
    ]
}

pub(super) fn basis_counter_values(bundle: &QueryBasisResultBundle) -> Vec<String> {
    let context = bundle.context();
    let execution = bundle.execution();
    let metadata: &QueryBasisMetadata = bundle.metadata();
    let prediction = metadata.prediction_report();

    vec![
        format!(
            "query_basis_bindings:{}",
            context.counters().query_basis_binding_count()
        ),
        format!(
            "historical_basis_lookups:{}",
            context.counters().historical_basis_lookup_count()
        ),
        format!(
            "comparison_basis_lookups:{}",
            context.counters().comparison_basis_lookup_count()
        ),
        format!(
            "materialization_path_compatibility_checks:{}",
            context
                .counters()
                .materialization_path_compatibility_check_count()
        ),
        format!(
            "basis_binding_width:{}",
            context.counters().basis_binding_width()
        ),
        format!(
            "historical_lookup_width:{}",
            context.counters().historical_lookup_width()
        ),
        format!(
            "comparison_binding_width:{}",
            prediction
                .map(|value| value.comparison_binding_width())
                .unwrap_or(0)
        ),
        format!(
            "comparison_scope_width:{}",
            context.counters().comparison_scope_width()
        ),
        format!(
            "diff_input_breadth:{}",
            context.counters().diff_input_breadth()
        ),
        format!(
            "diff_change_set_row_width:{}",
            context.counters().diff_change_set_row_width()
        ),
        format!("denial_width:{}", context.counters().denial_width()),
        format!(
            "unsupported_denials:{}",
            context.counters().unsupported_basis_denial_count()
        ),
        format!(
            "basis_substitution_denials:{}",
            context.counters().basis_substitution_denial_count()
        ),
        format!(
            "comparison_broadening_denials:{}",
            context.counters().comparison_broadening_denial_count()
        ),
        format!(
            "historical_broadening_denials:{}",
            context.counters().historical_broadening_denial_count()
        ),
        format!(
            "predicted_comparison_width:{}",
            prediction
                .map(|value| value.comparison_row_width())
                .unwrap_or(0)
        ),
        "realized_comparison_width:0".to_string(),
        "metadata_attachment_width:1".to_string(),
        format!(
            "query_context_execution_count:{}",
            execution.counters().context_execution_count()
        ),
        "query_context_metadata_attachment_count:1".to_string(),
        format!(
            "query_context_executor_rediscovery:{}",
            execution.counters().executor_rediscovery_count()
        ),
        format!(
            "basis_rediscovery:{}",
            context.counters().basis_rediscovery_count()
        ),
        format!(
            "historical_path_rediscovery:{}",
            context.counters().historical_path_rediscovery_count()
        ),
        format!(
            "comparison_family_rediscovery:{}",
            context.counters().comparison_family_rediscovery_count()
        ),
    ]
}

pub(super) fn diff_counter_values(
    bundle: &QueryDiffResultBundle,
    left_execution_count: usize,
    right_execution_count: usize,
    executor_rediscovery_count: usize,
) -> Vec<String> {
    let context = bundle.context();
    let metadata: &DiffQueryMetadata = bundle.metadata();

    vec![
        format!(
            "query_basis_bindings:{}",
            context.left().counters().query_basis_binding_count()
                + context.right().counters().query_basis_binding_count()
        ),
        format!(
            "historical_basis_lookups:{}",
            context.left().counters().historical_basis_lookup_count()
                + context.right().counters().historical_basis_lookup_count()
        ),
        format!(
            "comparison_basis_lookups:{}",
            context.counters().comparison_basis_lookup_count()
        ),
        format!(
            "materialization_path_compatibility_checks:{}",
            context
                .left()
                .counters()
                .materialization_path_compatibility_check_count()
                + context
                    .right()
                    .counters()
                    .materialization_path_compatibility_check_count()
        ),
        format!(
            "basis_binding_width:{}",
            context.left().counters().basis_binding_width()
                + context.right().counters().basis_binding_width()
        ),
        format!(
            "historical_lookup_width:{}",
            context.left().counters().historical_lookup_width()
                + context.right().counters().historical_lookup_width()
        ),
        format!(
            "comparison_binding_width:{}",
            metadata.prediction_report().comparison_binding_width()
        ),
        format!(
            "comparison_scope_width:{}",
            context.counters().comparison_scope_width()
        ),
        format!(
            "diff_input_breadth:{}",
            context.counters().diff_input_breadth()
        ),
        format!(
            "diff_change_set_row_width:{}",
            bundle.change_set().rows().len()
        ),
        format!("denial_width:{}", context.counters().denial_width()),
        format!(
            "unsupported_denials:{}",
            context.counters().unsupported_basis_denial_count()
        ),
        format!(
            "basis_substitution_denials:{}",
            context.counters().basis_substitution_denial_count()
        ),
        format!(
            "comparison_broadening_denials:{}",
            context.counters().comparison_broadening_denial_count()
        ),
        format!(
            "historical_broadening_denials:{}",
            context.counters().historical_broadening_denial_count()
        ),
        format!(
            "predicted_comparison_width:{}",
            metadata.prediction_report().comparison_row_width()
        ),
        format!(
            "realized_comparison_width:{}",
            bundle.change_set().rows().len()
        ),
        "metadata_attachment_width:1".to_string(),
        format!(
            "query_context_execution_count:{}",
            left_execution_count + right_execution_count
        ),
        "query_context_metadata_attachment_count:1".to_string(),
        format!(
            "query_context_executor_rediscovery:{}",
            executor_rediscovery_count
        ),
        format!(
            "basis_rediscovery:{}",
            context.counters().basis_rediscovery_count()
        ),
        format!(
            "historical_path_rediscovery:{}",
            context.counters().historical_path_rediscovery_count()
        ),
        format!(
            "comparison_family_rediscovery:{}",
            context.counters().comparison_family_rediscovery_count()
        ),
    ]
}
