pub(crate) use {worth_store::physical_runtime::MediaAdmissionInspectionCause};
pub(crate) use {worth_store_physical_backend::MediaCounterSnapshot, worth_store_physical_backend::MediaCounterTerminal, worth_store_physical_backend::MediaOperationRole};

pub(crate) use {crate::child_dispatch::emit, crate::child_dispatch::ChildReport};

pub(crate) fn emit_counter_report(
    outcome: &str,
    store: Option<String>,
    counters: MediaCounterSnapshot,
) {
    let mut fields = counter_fields(outcome, counters);
    if let Some(store) = store {
        fields.push(("store", store));
    }
    emit(&fields);
}

pub(crate) fn counter_fields(outcome: &str, counters: MediaCounterSnapshot) -> Vec<(&'static str, String)> {
    let fault = counters.first_fault_match();
    let fault_role = fault.map(|context| context.role());
    vec![
        ("outcome", outcome.into()),
        ("attempted", counters.attempted_operations().to_string()),
        ("completed", counters.completed_operations().to_string()),
        ("denied", counters.denied_before_effect().to_string()),
        ("partial", counters.partial_effects().to_string()),
        (
            "indeterminate",
            counters.indeterminate_effects().to_string(),
        ),
        ("requested_bytes", counters.requested_bytes().to_string()),
        ("completed_bytes", counters.completed_bytes().to_string()),
        ("retry_attempts", counters.retry_attempts().to_string()),
        ("file_syncs", counters.file_syncs().to_string()),
        ("directory_syncs", counters.directory_syncs().to_string()),
        ("replacements", counters.replacements().to_string()),
        ("deletions", counters.deletions().to_string()),
        ("cleanup", counters.cleanup_actions().to_string()),
        ("residue", counters.preserved_residue().to_string()),
        ("live_files", counters.live_file_handles().to_string()),
        (
            "live_directories",
            counters.live_directory_handles().to_string(),
        ),
        (
            "ownership_releases",
            counters.ownership_releases().to_string(),
        ),
        (
            "ownership_acquisitions",
            counters.ownership_acquisitions().to_string(),
        ),
        (
            "positioned_write_partial",
            counters
                .partial_effects_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite)
                .to_string(),
        ),
        (
            "directory_sync_denied",
            counters
                .denied_before_effect_for(worth_store_physical_backend::MediaOperationRole::SynchronizeDirectoryPublication)
                .to_string(),
        ),
        ("conserved", counters.is_conserved().to_string()),
        ("fault_matches", counters.fault_matches().to_string()),
        (
            "fault_terminal",
            counters
                .first_fault_terminal()
                .map_or("none", terminal_name)
                .into(),
        ),
        (
            "fault_role",
            fault
                .map_or("none", |context| context.role().metric_name())
                .into(),
        ),
        (
            "fault_ordinal",
            fault
                .map_or(0, |context| context.role_ordinal())
                .to_string(),
        ),
        (
            "fault_operation",
            fault
                .and_then(|context| context.operation())
                .map_or_else(|| "none".into(), |identity| identity.value().to_string()),
        ),
        (
            "fault_requested_bytes",
            fault
                .map_or(0, |context| context.requested_bytes())
                .to_string(),
        ),
        (
            "fault_completed_bytes",
            counters
                .first_fault_completed_bytes()
                .unwrap_or(0)
                .to_string(),
        ),
        (
            "fault_handle",
            fault.and_then(|context| context.handle()).map_or_else(
                || "none".into(),
                |identity| identity.generation().to_string(),
            ),
        ),
        (
            "fault_role_attempts",
            fault_role
                .map_or(0, |role| counters.attempts_for(role))
                .to_string(),
        ),
        (
            "fault_role_completed",
            fault_role
                .map_or(0, |role| counters.completed_operations_for(role))
                .to_string(),
        ),
        (
            "fault_role_denied",
            fault_role
                .map_or(0, |role| counters.denied_before_effect_for(role))
                .to_string(),
        ),
        (
            "fault_role_partial",
            fault_role
                .map_or(0, |role| counters.partial_effects_for(role))
                .to_string(),
        ),
        (
            "fault_role_indeterminate",
            fault_role
                .map_or(0, |role| counters.indeterminate_effects_for(role))
                .to_string(),
        ),
        ("counter_projection", exact_counter_projection(counters)),
    ]
}

pub(crate) fn exact_counter_projection(counters: MediaCounterSnapshot) -> String {
    let mut values = vec![
        counters.attempted_operations(),
        counters.completed_operations(),
        counters.denied_before_effect(),
        counters.partial_effects(),
        counters.indeterminate_effects(),
        counters.requested_bytes(),
        counters.completed_bytes(),
        counters.eof_observations(),
        counters.retry_attempts(),
        counters.listing_batches(),
        counters.listing_entries(),
        counters.qualification_transactions(),
        counters.ownership_attempts(),
        counters.ownership_acquisitions(),
        counters.ownership_contentions(),
        counters.ownership_releases(),
        counters.file_syncs(),
        counters.directory_syncs(),
        counters.replacements(),
        counters.deletions(),
        counters.file_opens(),
        counters.file_creates(),
        counters.file_closes(),
        counters.live_file_handles(),
        counters.peak_file_handles(),
        counters.directory_opens(),
        counters.directory_closes(),
        counters.live_directory_handles(),
        counters.peak_directory_handles(),
        counters.confinement_denials(),
        counters.stale_handle_denials(),
        counters.unsupported_capabilities(),
        counters.cleanup_actions(),
        counters.preserved_residue(),
        counters.peak_request_width_bytes(),
        counters.explicit_heap_allocation_events(),
        counters.requested_heap_capacity_bytes(),
        counters.fault_matches(),
    ];
    for role in worth_store_physical_backend::MediaOperationRole::ALL {
        values.extend([
            counters.attempts_for(role),
            counters.completed_operations_for(role),
            counters.denied_before_effect_for(role),
            counters.partial_effects_for(role),
            counters.indeterminate_effects_for(role),
            counters.requested_bytes_for(role),
            counters.completed_bytes_for(role),
        ]);
    }
    values
        .into_iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn terminal_name(terminal: MediaCounterTerminal) -> &'static str {
    match terminal {
        worth_store_physical_backend::MediaCounterTerminal::Completed => "completed",
        worth_store_physical_backend::MediaCounterTerminal::DeniedBeforeEffect => "denied",
        worth_store_physical_backend::MediaCounterTerminal::PartialEffect => "partial",
        worth_store_physical_backend::MediaCounterTerminal::IndeterminateEffect => "indeterminate",
    }
}

pub(crate) fn assert_counter_conservation(report: &ChildReport) {
    assert_eq!(report.value("conserved"), "true");
    assert_eq!(
        report.number("attempted"),
        report.number("completed")
            + report.number("denied")
            + report.number("partial")
            + report.number("indeterminate")
    );
    assert!(report.number("completed_bytes") <= report.number("requested_bytes"));
}

pub(crate) fn inspection_counters(cause: &MediaAdmissionInspectionCause) -> MediaCounterSnapshot {
    match cause {
        worth_store::physical_runtime::MediaAdmissionInspectionCause::BackendFailure(failure) => *failure.counters(),
    }
}