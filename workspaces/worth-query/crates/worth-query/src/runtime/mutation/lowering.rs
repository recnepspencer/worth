use super::{
    WorthQueryAspectMutationOperation, WorthQueryAspectMutationOperationKind,
    WorthQueryAuthoredAspectMutation, WorthQuerySymbolicAspectReference,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::{
    WorthQueryAspectTouch, WorthQueryMutationSymbolIdentity, WorthQueryWriteCommand,
};

pub(crate) fn command_declared_aspect_touches(
    command: &WorthQueryWriteCommand,
) -> Vec<crate::runtime::WorthQueryAspectTouch> {
    command_declared_aspect_operations(command)
        .into_iter()
        .map(|operation| operation.aspect_touch().clone())
        .collect()
}

pub(crate) fn command_declared_aspect_operations(
    command: &WorthQueryWriteCommand,
) -> Vec<WorthQueryAspectMutationOperation> {
    match command {
        WorthQueryWriteCommand::InsertAspects {
            aspects,
            symbolic_aspect_references,
            ..
        } => aspects
            .iter()
            .map(WorthQueryAuthoredAspectMutation::declared_operation)
            .chain(
                symbolic_aspect_references
                    .iter()
                    .map(symbolic_aspect_reference_operation),
            )
            .collect(),
        WorthQueryWriteCommand::UpdateAspects { aspects, .. }
        | WorthQueryWriteCommand::UpdateExistingAspects { aspects, .. }
        | WorthQueryWriteCommand::AssertExistingAspects { aspects, .. }
        | WorthQueryWriteCommand::VerifyExistingAspects { aspects, .. }
        | WorthQueryWriteCommand::UpdateSymbolicAspects { aspects, .. } => aspects
            .iter()
            .map(WorthQueryAuthoredAspectMutation::declared_operation)
            .collect(),
        WorthQueryWriteCommand::VerifyThenUpdateExistingAspects {
            aspects,
            symbolic_aspect_references,
            ..
        } => aspects
            .iter()
            .map(WorthQueryAuthoredAspectMutation::declared_operation)
            .chain(
                symbolic_aspect_references
                    .iter()
                    .map(symbolic_aspect_reference_operation),
            )
            .collect(),
        WorthQueryWriteCommand::DeleteAspects {
            touched_aspects, ..
        }
        | WorthQueryWriteCommand::VerifyThenDeleteExistingAspects {
            touched_aspects, ..
        }
        | WorthQueryWriteCommand::DeleteExistingAspects {
            touched_aspects, ..
        }
        | WorthQueryWriteCommand::DeleteSymbolicAspects {
            touched_aspects, ..
        } => touched_aspects
            .iter()
            .map(|touch| {
                WorthQueryAspectMutationOperation::from_touch(
                    touch.clone(),
                    crate::runtime::WorthQueryAspectMutationOperationKind::Clear,
                )
            })
            .collect(),
        WorthQueryWriteCommand::UpdateAspect { aspect, .. } => vec![aspect.declared_operation()],
        WorthQueryWriteCommand::Delete { .. } => Vec::new(),
    }
}

pub(crate) fn command_declared_aspect_value_digest(
    command: &WorthQueryWriteCommand,
) -> Option<String> {
    command_declared_aspect_value_identity(command).map(|identity| identity.as_str().to_string())
}

pub(crate) fn command_declared_aspect_value_identity(
    command: &WorthQueryWriteCommand,
) -> Result<Option<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
    let aspects = match command {
        WorthQueryWriteCommand::InsertAspects {
            aspects,
            symbolic_aspect_references,
            ..
        } => {
            let rows =
                aspects
                    .iter()
                    .map(|aspect| {
                        declared_aspect_value_digest_row(
                            "declared",
                            aspect.aspect_touch(),
                            aspect.clears_existing_value(),
                            aspect.terminal_digest_material(),
                        )
                    })
                    .chain(symbolic_aspect_references.iter().map(|reference| {
                        symbolic_aspect_reference_digest_row("symbolic", reference)
                    }))
                    .collect::<Result<Vec<_>, _>>()?;
            return Ok(Some(declared_aspect_value_identity(rows)?));
        }
        WorthQueryWriteCommand::UpdateAspects { aspects, .. }
        | WorthQueryWriteCommand::UpdateExistingAspects { aspects, .. }
        | WorthQueryWriteCommand::AssertExistingAspects { aspects, .. }
        | WorthQueryWriteCommand::VerifyExistingAspects { aspects, .. }
        | WorthQueryWriteCommand::UpdateSymbolicAspects { aspects, .. } => aspects,
        WorthQueryWriteCommand::VerifyThenUpdateExistingAspects {
            asserted_aspects,
            aspects,
            symbolic_aspect_references,
            ..
        } => {
            let rows = asserted_aspects
                .iter()
                .map(|aspect| {
                    declared_aspect_value_digest_row(
                        "assert",
                        aspect.aspect_touch(),
                        aspect.clears_existing_value(),
                        aspect.terminal_digest_material(),
                    )
                })
                .chain(aspects.iter().map(|aspect| {
                    declared_aspect_value_digest_row(
                        "update",
                        aspect.aspect_touch(),
                        aspect.clears_existing_value(),
                        aspect.terminal_digest_material(),
                    )
                }))
                .chain(symbolic_aspect_references.iter().map(|reference| {
                    symbolic_aspect_reference_digest_row("update-symbolic", reference)
                }))
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(Some(declared_aspect_value_identity(rows)?));
        }
        WorthQueryWriteCommand::VerifyThenDeleteExistingAspects {
            asserted_aspects,
            touched_aspects,
            ..
        } => {
            let rows = asserted_aspects
                .iter()
                .map(|aspect| {
                    declared_aspect_value_digest_row(
                        "assert",
                        aspect.aspect_touch(),
                        aspect.clears_existing_value(),
                        aspect.terminal_digest_material(),
                    )
                })
                .chain(
                    touched_aspects
                        .iter()
                        .map(|touch| touched_aspect_digest_row("delete", touch)),
                )
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(Some(declared_aspect_value_identity(rows)?));
        }
        WorthQueryWriteCommand::UpdateAspect { aspect, .. } => {
            return Ok(Some(declared_aspect_value_identity(vec![
                declared_aspect_value_digest_row(
                    "declared",
                    aspect.aspect_touch(),
                    aspect.clears_existing_value(),
                    aspect.terminal_digest_material(),
                )?,
            ])?));
        }
        WorthQueryWriteCommand::DeleteAspects { .. }
        | WorthQueryWriteCommand::DeleteExistingAspects { .. }
        | WorthQueryWriteCommand::DeleteSymbolicAspects { .. }
        | WorthQueryWriteCommand::Delete { .. } => return Ok(None),
    };
    Ok(Some(declared_aspect_value_identity(
        aspects
            .iter()
            .map(|aspect| {
                declared_aspect_value_digest_row(
                    "declared",
                    aspect.aspect_touch(),
                    aspect.clears_existing_value(),
                    aspect.terminal_digest_material(),
                )
            })
            .collect::<Result<Vec<_>, _>>()?,
    )?))
}

fn symbolic_aspect_reference_operation(
    reference: &WorthQuerySymbolicAspectReference,
) -> WorthQueryAspectMutationOperation {
    WorthQueryAspectMutationOperation::from_touch(
        reference.aspect_touch().clone(),
        WorthQueryAspectMutationOperationKind::Set,
    )
}

fn declared_aspect_value_digest_row(
    prefix: &str,
    aspect_touch: WorthQueryAspectTouch,
    clears_existing_value: bool,
    terminal_value_digest: String,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(
                WorthQueryEvidenceTag::new("role"),
                "declared-aspect-value-row",
            )
            .field_shape(WorthQueryEvidenceTag::new("prefix"), prefix)
            .field_value(
                WorthQueryEvidenceTag::new("aspect_touch"),
                aspect_touch.admitted_touch_digest_part(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("operation"),
                if clears_existing_value {
                    "clear"
                } else {
                    "set"
                },
            )
            .field_value(
                WorthQueryEvidenceTag::new("terminal_value"),
                terminal_value_digest,
            )
            .seal()?,
    )
}

fn symbolic_aspect_reference_digest_row(
    prefix: &'static str,
    reference: &WorthQuerySymbolicAspectReference,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    let symbol_identity = WorthQueryMutationSymbolIdentity::new(
        "symbolic-aspect-reference",
        reference.reference().symbol(),
    )?;
    let collection_identity = reference.reference().target_collection_identity();
    let mut identity =
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(
                WorthQueryEvidenceTag::new("role"),
                "symbolic-aspect-reference-row",
            )
            .field_shape(WorthQueryEvidenceTag::new("prefix"), prefix)
            .field_value(
                WorthQueryEvidenceTag::new("aspect_touch"),
                reference.aspect_touch().admitted_touch_digest_part(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                reference.family().as_str(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("symbol"),
                symbol_identity.evidence_identity(),
            );
    if let Some(collection) = collection_identity {
        identity = identity.field_evidence_identity(
            WorthQueryEvidenceTag::new("collection"),
            collection.evidence_identity(),
        );
    }
    Ok(identity.seal()?)
}

fn touched_aspect_digest_row(
    prefix: &'static str,
    aspect_touch: &WorthQueryAspectTouch,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(WorthQueryEvidenceTag::new("role"), "touched-aspect-row")
            .field_shape(WorthQueryEvidenceTag::new("prefix"), prefix)
            .field_value(
                WorthQueryEvidenceTag::new("aspect_touch"),
                aspect_touch.admitted_touch_digest_part(),
            )
            .seal()?,
    )
}

fn declared_aspect_value_identity(
    rows: Vec<WorthQueryEvidenceIdentity>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
            .field_shape(
                WorthQueryEvidenceTag::new("role"),
                "declared-aspect-value-digest",
            )
            .field_evidence_identity_sequence(WorthQueryEvidenceTag::new("row"), rows.iter())
            .seal()?,
    )
}
