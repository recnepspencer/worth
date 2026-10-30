pub(crate) use std::collections::BTreeMap;
pub(crate) use std::path::Path;

pub(crate) use crate::phase_eight_process::history::artifacts::collect_files;
pub(crate) use {
    crate::history::parent_oracle,
    crate::phase_eight_process::history::writer_history::expected_history::ExpectedWriterHistory,
};

/// Derive operation fates from the surviving physical bytes. The writer
/// receipt supplies the independent idempotency-to-record-to-payload binding,
/// but its fate byte is deliberately not read here.
pub(crate) fn classify_persisted_fates(
    expected: &ExpectedWriterHistory,
    root: &Path,
) -> Result<BTreeMap<[u8; 32], u8>, String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("canonicalize persisted-fate root: {error}"))?;
    let mut files = Vec::new();
    collect_files(&root, &root, &mut files)?;
    let selected_basis =
        crate::phase_eight_process::history::parent_oracle::selected_basis::select(&files)?;
    let current_records = crate::phase_eight_process::history::parent_oracle::canonical_membership::current_root_records(&files)?;
    crate::phase_eight_process::history::parent_oracle::in_flight::record_binding::require_bound_records(&files, expected.durable_bindings())?;
    let mut fates = std::collections::BTreeMap::new();

    for (idempotency, binding) in expected.durable_bindings() {
        let record = crate::phase_eight_process::history::parent_oracle::canonical_membership_placement::RecordIdentity {
            allocation_epoch: binding.allocation_epoch,
            ordinal: binding.ordinal,
        };
        let fate = if current_records.get(&record) == Some(&binding.payload) {
            1
        } else if crate::history::parent_oracle::in_flight::classify(
            &selected_basis,
            idempotency,
            &binding.payload,
        )?
        .0
        {
            2
        } else {
            4
        };
        fates.insert(*idempotency, fate);
    }

    let no_effect = expected
        .no_effect_idempotency()
        .ok_or_else(|| "missing no-effect identity binding".to_owned())?;
    if !crate::phase_eight_process::history::parent_oracle::terminal::contains_persisted_no_effect_terminal(&selected_basis, &no_effect)? {
        return Err("persisted no-effect terminal fact is absent".to_owned());
    }
    fates.insert(no_effect, 3);

    let dirty = expected
        .dirty_idempotency()
        .ok_or_else(|| "missing dirty idempotency binding".to_owned())?;
    let dirty_fate = if crate::phase_eight_process::history::parent_oracle::canonical_membership::require_current_root_membership_with_unresolved_payload(
        &files,
        expected.durable_bindings(),
        &dirty,
        expected.in_flight_payload(),
    )? {
        2
    } else {
        4
    };
    fates.insert(dirty, dirty_fate);
    Ok(fates)
}
