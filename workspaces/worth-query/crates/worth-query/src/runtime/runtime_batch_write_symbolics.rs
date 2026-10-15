use super::*;
pub(super) fn admit_atomic_batch_symbolic_references(
    planned_symbolic_targets: &BTreeMap<
        WorthQuerySameBatchSymbolicTargetKey,
        WorthQuerySameBatchSymbolicTarget,
    >,
    command: &WorthQueryWriteCommand,
) -> Result<(), WorthQueryRuntimeError> {
    if let Some(reference) = command.symbolic_target_reference()
        && !matches!(command.mutation_family(), WorthQueryMutationFamily::Insert)
    {
        resolve_same_batch_symbolic_target(planned_symbolic_targets, reference)?;
    }
    for reference in command.symbolic_aspect_references() {
        resolve_same_batch_symbolic_target(planned_symbolic_targets, reference.reference())?;
    }
    Ok(())
}

pub(super) fn record_planned_same_batch_symbolic_target(
    planned_symbolic_targets: &mut BTreeMap<
        WorthQuerySameBatchSymbolicTargetKey,
        WorthQuerySameBatchSymbolicTarget,
    >,
    command: &WorthQueryWriteCommand,
) {
    if !matches!(command.mutation_family(), WorthQueryMutationFamily::Insert) {
        return;
    }
    let Some(reference) = command.symbolic_target_reference() else {
        return;
    };
    let planned_identity = crate::memory_workspace::admit_authored_entity_label(format!(
        "planned-symbolic:{}",
        reference.symbol()
    ));
    planned_symbolic_targets.insert(
        WorthQuerySameBatchSymbolicTargetKey::from_reference(reference),
        WorthQuerySameBatchSymbolicTarget::new(
            planned_identity,
            command.declared_collection_identity(),
        ),
    );
}

pub(super) fn symbolic_aspect_resolution_evidence_for_command(
    symbolic_targets: &BTreeMap<
        WorthQuerySameBatchSymbolicTargetKey,
        WorthQuerySameBatchSymbolicTarget,
    >,
    command: &WorthQueryWriteCommand,
) -> Result<Vec<WorthQuerySymbolicAspectResolutionEvidence>, WorthQueryRuntimeError> {
    symbolic_aspect_resolution_evidence_for_references(
        symbolic_targets,
        command.symbolic_aspect_references(),
    )
}

pub(super) fn symbolic_aspect_resolution_evidence_for_references(
    symbolic_targets: &BTreeMap<
        WorthQuerySameBatchSymbolicTargetKey,
        WorthQuerySameBatchSymbolicTarget,
    >,
    symbolic_aspect_references: &[WorthQuerySymbolicAspectReference],
) -> Result<Vec<WorthQuerySymbolicAspectResolutionEvidence>, WorthQueryRuntimeError> {
    symbolic_aspect_references
        .iter()
        .map(|reference| {
            let resolved_target =
                resolve_same_batch_symbolic_target(symbolic_targets, reference.reference())?;
            Ok(WorthQuerySymbolicAspectResolutionEvidence::from_reference(
                reference,
                resolved_target.entity_identity(),
            ))
        })
        .collect()
}
