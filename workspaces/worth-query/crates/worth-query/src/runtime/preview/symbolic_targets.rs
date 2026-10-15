use super::*;

pub(super) fn admit_preview_batch_symbolic_references(
    commands: &[WorthQueryWriteCommand],
) -> Result<(), WorthQueryRuntimeError> {
    let mut planned_symbolic_targets =
        BTreeMap::<WorthQuerySameBatchSymbolicTargetKey, WorthQuerySameBatchSymbolicTarget>::new();
    for command in commands {
        if let Some(reference) = command.symbolic_target_reference()
            && command.mutation_family() != crate::runtime::WorthQueryMutationFamily::Insert
        {
            resolve_preview_symbolic_target(&planned_symbolic_targets, reference)?;
        }
        for reference in command.symbolic_aspect_references() {
            resolve_preview_symbolic_target(&planned_symbolic_targets, reference.reference())?;
        }
        record_planned_preview_symbolic_target(&mut planned_symbolic_targets, command);
    }
    Ok(())
}

fn record_planned_preview_symbolic_target(
    planned_symbolic_targets: &mut BTreeMap<
        WorthQuerySameBatchSymbolicTargetKey,
        WorthQuerySameBatchSymbolicTarget,
    >,
    command: &WorthQueryWriteCommand,
) {
    if command.mutation_family() != crate::runtime::WorthQueryMutationFamily::Insert {
        return;
    }
    let Some(reference) = command.symbolic_target_reference() else {
        return;
    };
    let planned_identity = crate::memory_workspace::admit_authored_entity_label(format!(
        "planned-preview-symbolic:{}",
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

pub(super) fn record_preview_symbolic_target(
    symbolic_targets: &mut BTreeMap<
        WorthQuerySameBatchSymbolicTargetKey,
        WorthQuerySameBatchSymbolicTarget,
    >,
    reference: &WorthQuerySymbolicTargetReference,
    receipt: &WorthQueryWriteReceipt,
) {
    if receipt.mutation_family() != crate::runtime::WorthQueryMutationFamily::Insert {
        return;
    }
    let Some(target_entity_identity) = receipt.target_entity_identity() else {
        return;
    };
    symbolic_targets.insert(
        WorthQuerySameBatchSymbolicTargetKey::from_reference(reference),
        WorthQuerySameBatchSymbolicTarget::new(
            target_entity_identity.clone(),
            receipt.target_collection_identity().cloned(),
        ),
    );
}

pub(super) fn resolve_preview_symbolic_target(
    symbolic_targets: &BTreeMap<
        WorthQuerySameBatchSymbolicTargetKey,
        WorthQuerySameBatchSymbolicTarget,
    >,
    reference: &WorthQuerySymbolicTargetReference,
) -> Result<WorthQuerySameBatchSymbolicTarget, WorthQueryRuntimeError> {
    let target_key = WorthQuerySameBatchSymbolicTargetKey::from_reference(reference);
    let Some(resolved_target) = symbolic_targets.get(&target_key) else {
        return Err(WorthQueryRuntimeError::MutationTargetReferenceDenied(
            WorthQuerySymbolicTargetReferenceDenial::new(
                reference,
                WorthQuerySymbolicTargetReferenceDenialKind::UnresolvedSameBatchTarget,
                format!(
                    "same-batch symbolic target `{}` was not declared earlier in this preview batch",
                    reference.symbol()
                ),
            ),
        ));
    };
    if let Some(expected_collection) = reference.target_collection_identity()
        && resolved_target
            .target_collection_identity()
            .is_none_or(|resolved_collection| {
                !resolved_collection.same_target_collection_as(expected_collection)
            })
    {
        return Err(WorthQueryRuntimeError::MutationTargetReferenceDenied(
                WorthQuerySymbolicTargetReferenceDenial::new(
                    reference,
                    WorthQuerySymbolicTargetReferenceDenialKind::CollectionMismatch,
                    format!(
                        "same-batch symbolic target `{}` resolved to collection `{}`, not `{expected_collection}`",
                        reference.symbol(),
                        resolved_target
                            .target_collection_identity()
                            .map(|collection| collection.as_str())
                            .unwrap_or("unknown"),
                        expected_collection = expected_collection.as_str(),
                    ),
                ),
            ));
    }
    Ok(resolved_target.clone())
}
