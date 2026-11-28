use worth_signal::facade::{
    AsyncNodeCapabilityDeclaration, ResourceNodeDeclaration, ResourceRetryBudgetScope,
};

pub(super) fn request_response_basis(
    declaration_identity: &str,
    declaration: &ResourceNodeDeclaration,
) -> String {
    format!(
        concat!(
            "bridge-async-source|family=request-response|id={}|",
            "lifecycle={:?}|retry={:?}|timeout={:?}|cancellation={:?}|stale_after={:?}|",
            "supersession={:?}|revalidation={:?}|observation={:?}|output_continuity={:?}|",
            "retention={:?}|diagnostics={:?}|replay={:?}|cancellation_grace={}|",
            "dependent_node_count={}|retry_max_attempts={}|retry_jitter={}|retry_budget_scope={}|",
            "retry_budget_limit={}|payload_contract_id={}|payload_max_bytes={}"
        ),
        declaration_identity,
        declaration.lifecycle_policy(),
        declaration.retry_policy(),
        declaration.timeout_policy(),
        declaration.cancellation_policy(),
        declaration.stale_after_policy(),
        declaration.supersession_policy(),
        declaration.revalidation_policy(),
        declaration.observation_policy(),
        declaration.output_continuity_policy(),
        declaration.retention_policy(),
        declaration.diagnostics_policy(),
        declaration.replay_policy(),
        optional_debug(declaration.cancellation_grace_period()),
        declaration.declared_dependent_cancellation_nodes().len(),
        optional_display(declaration.retry_max_attempts()),
        optional_debug(declaration.retry_deterministic_jitter()),
        optional_retry_scope(declaration.retry_budget_scope()),
        optional_display(declaration.retry_budget_limit()),
        declaration.payload_contract().id().get(),
        optional_display(declaration.payload_contract().max_payload_bytes()),
    )
}

pub(super) fn subscription_backed_basis(
    declaration_identity: &str,
    declaration: &AsyncNodeCapabilityDeclaration,
) -> String {
    format!(
        concat!(
            "bridge-async-source|family=subscription-backed|id={}|",
            "lifecycle={:?}|retry={:?}|timeout={:?}|cancellation={:?}|stale_after={:?}|",
            "supersession={:?}|revalidation={:?}|observation={:?}|output_continuity={:?}|",
            "retention={:?}|diagnostics={:?}|replay={:?}|cancellation_grace={}|",
            "dependent_node_count={}|retry_max_attempts={}|retry_jitter={}|retry_budget_scope={}|",
            "retry_budget_limit={}|payload_contract_id={}|payload_max_bytes={}"
        ),
        declaration_identity,
        declaration.lifecycle_policy(),
        declaration.retry_policy(),
        declaration.timeout_policy(),
        declaration.cancellation_policy(),
        declaration.stale_after_policy(),
        declaration.supersession_policy(),
        declaration.revalidation_policy(),
        declaration.observation_policy(),
        declaration.output_continuity_policy(),
        declaration.retention_policy(),
        declaration.diagnostics_policy(),
        declaration.replay_policy(),
        optional_debug(declaration.cancellation_grace_period()),
        declaration.declared_dependent_cancellation_nodes().len(),
        optional_display(declaration.retry_max_attempts()),
        optional_debug(declaration.retry_deterministic_jitter()),
        optional_retry_scope(declaration.retry_budget_scope()),
        optional_display(declaration.retry_budget_limit()),
        declaration.payload_contract().id().get(),
        optional_display(declaration.payload_contract().max_payload_bytes()),
    )
}

fn optional_debug<T: std::fmt::Debug>(value: Option<T>) -> String {
    match value {
        Some(value) => format!("{value:?}"),
        None => "none".to_owned(),
    }
}

fn optional_display<T: std::fmt::Display>(value: Option<T>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "none".to_owned(),
    }
}

fn optional_retry_scope(value: Option<ResourceRetryBudgetScope>) -> String {
    match value {
        Some(value) => format!("{value:?}"),
        None => "none".to_owned(),
    }
}
