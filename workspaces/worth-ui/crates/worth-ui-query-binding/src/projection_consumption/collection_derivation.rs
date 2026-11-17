use worth_foundational::facade::{AspectValue, InternedString};
use worth_query::facade::installed::{collection, operation};

use super::{
    UiCollectionCompleteness, UiCollectionContinuation, UiCollectionProjectionChange,
    UiCollectionProjectionDelivery, UiCollectionProjectionFactReceipt,
    UiCollectionProjectionRowReference, UiCollectionProjectionTextRow, UiCollectionProjectionValue,
    UiCollectionProjectionWorkCounters, UiNativeTextValue, UiPresentProjection,
    UiProjectionAvailability, UiProjectionFactReceipt, UiProjectionFactStopKind,
    UiProjectionFactStopReceipt, UiProjectionUnavailableKind, UiProjectionUnavailableReceipt,
};

pub(crate) struct UiCollectionDerivationContext<'a> {
    pub binding: &'a crate::UiCollectionProjectionBinding,
    pub consumer: &'a collection::WorthQueryCollectionConsumerWindow,
    pub text_accesses: &'a [crate::application_binding::WorthUiCollectionTextNativeAccess],
    pub application_item_key_access:
        Option<&'a crate::application_binding::WorthUiCollectionTextNativeAccess>,
    pub budget: crate::UiCollectionProjectionBudget,
}

pub(crate) fn derive_initial_collection_projection(
    context: UiCollectionDerivationContext<'_>,
) -> Result<
    UiCollectionProjectionFactReceipt,
    worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial,
> {
    let rows = context.consumer.rows().iter().collect::<Vec<_>>();
    derive_collection_projection(
        context,
        &rows,
        UiCollectionProjectionDelivery::Snapshot,
        Box::<[UiCollectionProjectionChange]>::default(),
    )
}

pub(crate) fn derive_collection_projection(
    context: UiCollectionDerivationContext<'_>,
    rows: &[&collection::WorthQueryCollectionRowHandle],
    delivery: UiCollectionProjectionDelivery,
    changes: Box<[UiCollectionProjectionChange]>,
) -> Result<
    UiCollectionProjectionFactReceipt,
    worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial,
> {
    let result_identity = context.consumer.result_generation_identity_evidence()?;
    let core = UiProjectionFactReceipt::admitted(super::UiProjectionFactReceiptInput {
        projection_identity: context.binding.view_identity().clone(),
        observation_order: context.binding.issue_observation_order(),
        query_world_identity: context.binding.query_world_identity().clone(),
        binding_identity: context.consumer.binding_identity_evidence()?,
        source_generation_identity: context.consumer.source_generation_identity_evidence()?,
        result_generation_identity: result_identity.clone(),
    });
    let mut work = UiCollectionProjectionWorkCounters::default();
    for access in context.text_accesses {
        work.record_key_resolution(access.resolution_counters());
    }
    if let Some(access) = context.application_item_key_access {
        work.record_key_resolution(access.resolution_counters());
    }
    let availability = if changes.len() > context.budget.max_change_operations() {
        stopped(
            UiProjectionFactStopKind::BudgetExceeded,
            &result_identity,
            "the collection change-operation budget is exhausted",
        )
    } else if changes
        .iter()
        .any(|change| matches!(change, UiCollectionProjectionChange::ResetRequired { .. }))
    {
        stopped(
            UiProjectionFactStopKind::ResetRequired,
            &result_identity,
            "Query requires explicit collection replacement",
        )
    } else {
        derive_availability(&context, rows, &result_identity, &mut work)?
    };
    Ok(UiCollectionProjectionFactReceipt::admitted(
        core,
        delivery,
        availability,
        work,
        changes,
    ))
}

fn derive_availability(
    context: &UiCollectionDerivationContext<'_>,
    rows: &[&collection::WorthQueryCollectionRowHandle],
    result_identity: &worth_query::facade::runtime::WorthQueryEvidenceIdentity,
    work: &mut UiCollectionProjectionWorkCounters,
) -> Result<
    UiProjectionAvailability<UiCollectionProjectionValue>,
    worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial,
> {
    Ok(match context.consumer.result_state() {
        operation::WorthQueryOperationResultState::Pending => {
            UiProjectionAvailability::Unavailable(UiProjectionUnavailableReceipt::query_issued(
                UiProjectionUnavailableKind::Pending,
                result_identity.clone(),
            ))
        }
        operation::WorthQueryOperationResultState::Violation => stopped(
            UiProjectionFactStopKind::PayloadShapeMismatch,
            result_identity,
            "Query reported a violating collection result",
        ),
        operation::WorthQueryOperationResultState::Ready
        | operation::WorthQueryOperationResultState::Advisory
        | operation::WorthQueryOperationResultState::Partial => {
            derive_present(context, rows, result_identity, work)?
        }
    })
}

fn derive_present(
    context: &UiCollectionDerivationContext<'_>,
    selected_rows: &[&collection::WorthQueryCollectionRowHandle],
    result_identity: &worth_query::facade::runtime::WorthQueryEvidenceIdentity,
    work: &mut UiCollectionProjectionWorkCounters,
) -> Result<
    UiProjectionAvailability<UiCollectionProjectionValue>,
    worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial,
> {
    let completeness =
        if context.consumer.result_state() == operation::WorthQueryOperationResultState::Partial {
            UiCollectionCompleteness::Partial
        } else {
            UiCollectionCompleteness::Complete
        };
    if completeness == UiCollectionCompleteness::Partial
        && context.binding.requirement().requires_complete_result()
    {
        return Ok(stopped(
            UiProjectionFactStopKind::PayloadShapeMismatch,
            result_identity,
            "the binding requires a complete Query result",
        ));
    }
    let continuation = match context.consumer.continuation().identity_evidence()? {
        Some(_identity) if !context.binding.requirement().permits_continuation() => {
            return Ok(stopped(
                UiProjectionFactStopKind::PayloadShapeMismatch,
                result_identity,
                "the binding forbids Query continuation",
            ));
        }
        Some(_) if context.budget.max_continuation_operations() == 0 => {
            return Ok(stopped(
                UiProjectionFactStopKind::BudgetExceeded,
                result_identity,
                "the collection continuation budget is exhausted",
            ));
        }
        Some(identity) => {
            work.record_continuation();
            Some(UiCollectionContinuation::query_issued(identity))
        }
        None => None,
    };
    let rows = match derive_rows(context, selected_rows, result_identity, work) {
        Ok(rows) => rows,
        Err(stop) => return Ok(UiProjectionAvailability::Stopped(*stop)),
    };
    Ok(UiProjectionAvailability::Present(
        UiPresentProjection::Current(UiCollectionProjectionValue::admitted(
            rows,
            completeness,
            continuation,
        )),
    ))
}

fn derive_rows(
    context: &UiCollectionDerivationContext<'_>,
    selected_rows: &[&collection::WorthQueryCollectionRowHandle],
    result_identity: &worth_query::facade::runtime::WorthQueryEvidenceIdentity,
    work: &mut UiCollectionProjectionWorkCounters,
) -> Result<Box<[UiCollectionProjectionTextRow]>, Box<UiProjectionFactStopReceipt>> {
    let mut rows = Vec::with_capacity(selected_rows.len());
    for row in selected_rows {
        work.visit_row();
        let mut values = Vec::with_capacity(context.text_accesses.len());
        for access in context.text_accesses {
            let fact = context
                .consumer
                .native_value(row, access.key())
                .map_err(|_| {
                    Box::new(stop_receipt(
                        UiProjectionFactStopKind::WrongWorld,
                        result_identity,
                        "Query rejected the collection row or native key authority",
                    ))
                })?;
            let text = match fact.native_value().scalar() {
                Some(AspectValue::String(InternedString::Raw(text))) => text.clone(),
                Some(AspectValue::String(InternedString::Stored(text))) => text.as_str().to_owned(),
                _ => {
                    return Err(Box::new(stop_receipt(
                        UiProjectionFactStopKind::NativeFamilyMismatch,
                        result_identity,
                        "the selected Query native value is not direct raw text",
                    )));
                }
            };
            if work.native_bytes_retained().saturating_add(text.len())
                > context.budget.max_native_bytes()
            {
                return Err(Box::new(stop_receipt(
                    UiProjectionFactStopKind::BudgetExceeded,
                    result_identity,
                    "the collection native-text byte budget is exhausted",
                )));
            }
            work.record_native_access(fact.counters(), text.len());
            values.push(UiNativeTextValue::from_raw(text));
        }
        let application_item_key = context
            .application_item_key_access
            .map(|access| derive_application_item_key(context, row, access, result_identity, work))
            .transpose()?;
        rows.push(UiCollectionProjectionTextRow::admitted(
            UiCollectionProjectionRowReference::query_issued(
                row.entity_identity().evidence_identity(),
            ),
            values,
            application_item_key,
        ));
    }
    Ok(rows.into_boxed_slice())
}

fn derive_application_item_key(
    context: &UiCollectionDerivationContext<'_>,
    row: &collection::WorthQueryCollectionRowHandle,
    access: &crate::application_binding::WorthUiCollectionTextNativeAccess,
    result_identity: &worth_query::facade::runtime::WorthQueryEvidenceIdentity,
    work: &mut UiCollectionProjectionWorkCounters,
) -> Result<core::num::NonZeroU64, Box<UiProjectionFactStopReceipt>> {
    let fact = context
        .consumer
        .native_value(row, access.key())
        .map_err(|_| {
            Box::new(stop_receipt(
                UiProjectionFactStopKind::WrongWorld,
                result_identity,
                "Query rejected the application item-key authority",
            ))
        })?;
    let Some(AspectValue::UInt64(value)) = fact.native_value().scalar() else {
        return Err(Box::new(stop_receipt(
            UiProjectionFactStopKind::NativeFamilyMismatch,
            result_identity,
            "the application item-key value is not unsigned64",
        )));
    };
    let Some(value) = core::num::NonZeroU64::new(*value) else {
        return Err(Box::new(stop_receipt(
            UiProjectionFactStopKind::PayloadShapeMismatch,
            result_identity,
            "the application item-key value is zero",
        )));
    };
    work.record_native_access(fact.counters(), core::mem::size_of::<u64>());
    Ok(value)
}

fn stopped(
    kind: UiProjectionFactStopKind,
    result_identity: &worth_query::facade::runtime::WorthQueryEvidenceIdentity,
    summary: &'static str,
) -> UiProjectionAvailability<UiCollectionProjectionValue> {
    UiProjectionAvailability::Stopped(stop_receipt(kind, result_identity, summary))
}

fn stop_receipt(
    kind: UiProjectionFactStopKind,
    result_identity: &worth_query::facade::runtime::WorthQueryEvidenceIdentity,
    summary: &'static str,
) -> UiProjectionFactStopReceipt {
    UiProjectionFactStopReceipt::query_issued(kind, result_identity.clone(), None, summary)
}
