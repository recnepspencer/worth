//! Restore the outcome frontier from actual latest-Native binding rows, never receipts.
use super::recorded_intent;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
    bootstrap::request_read_denial, schema_layout::WorthQueryProviderIdempotencyLayout,
    WorthQueryPrimaryGraphInstallationDenial as Denial,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_foundational::facade::{
    AspectValue, CanonicalDigestId, ContractValidatedAspectValueView, InternedString,
};
use worth_relational::facade::runtime::{RelationalRuntime, VisibilityProjectionView};

pub(in crate::domain_computation::primary_graph) fn readmit(
    native: &RelationalRuntime,
    view: &VisibilityProjectionView,
    layout: &WorthQueryProviderIdempotencyLayout,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let rows = view
        .bounded_entities_of_kind(execution, layout.entity_kind, usize::MAX)
        .map_err(request_read_denial::read)?
        .map_err(|_| denied("Home idempotency scan exceeded its bound"))?;
    let bytes = rows
        .records()
        .len()
        .checked_mul(std::mem::size_of::<([u8; 32], u64)>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(|| denied("Home idempotency inventory size overflow"))?;
    let _funding = execution
        .reserve_retained_memory(bytes)
        .map_err(|cause| request_read_denial::read(cause.into()))?;
    let mut identities = Vec::new();
    identities
        .try_reserve_exact(rows.records().len())
        .map_err(|_| denied("Home idempotency inventory allocation refused"))?;
    execution
        .run_native_with_checkpoint(|context| {
            for row in rows.records() {
                identities.push(admit_row(native, row, layout, context)?);
            }
            validate_distinct(&mut identities, context)?;
            restore_frontier(identities, context)?;
            Ok(())
        })
        .map_err(|cause| request_read_denial::read(cause.into()))?
        .0
}
fn denied(subject: impl Into<String>) -> Denial {
    Denial::new(Kind::CheckpointRecoveryRejected, subject)
}

fn admit_row(
    native: &RelationalRuntime,
    row: &worth_relational::facade::runtime::EntityReadRecord,
    layout: &WorthQueryProviderIdempotencyLayout,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<([u8; 32], u64), Denial> {
    context
        .checkpoint(1)
        .map_err(|stop| request_read_denial::read(WorkCeilingDenial::Stopped(stop).into()))?;
    let state = row
        .authoritative_aspect_state
        .as_ref()
        .ok_or_else(|| denied("Home idempotency state is missing"))?;
    let aspect = state
        .get(layout.key_locator.aspect().aspect_key())
        .ok_or_else(|| denied("Home idempotency aspect is missing"))?;
    let ContractValidatedAspectValueView::Struct(fields) = aspect.view() else {
        return Err(denied("Home idempotency aspect has foreign shape"));
    };
    let field = |locator: &worth_foundational::facade::AspectFieldLocator| {
        fields
            .get(
                locator
                    .field_path()
                    .fields()
                    .first()
                    .expect("installed field"),
            )
            .ok_or_else(|| denied("Home idempotency required field is missing"))
    };
    let AspectValue::String(InternedString::Raw(key)) = field(&layout.key_locator)? else {
        return Err(denied("Home idempotency key has foreign shape"));
    };
    let AspectValue::String(InternedString::Raw(intent)) = field(&layout.intent_locator)? else {
        return Err(denied("Home idempotency intent has foreign shape"));
    };
    context
        .checkpoint(
            u64::try_from(
                key.len()
                    .checked_add(intent.len())
                    .ok_or_else(|| denied("Home idempotency text length overflow"))?,
            )
            .map_err(|_| denied("Home idempotency text length overflow"))?,
        )
        .map_err(|stop| request_read_denial::read(WorkCeilingDenial::Stopped(stop).into()))?;
    if key.len() != 64
        || !key
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(denied("Home idempotency key is not canonical"));
    }
    let key = CanonicalDigestId::parse_hex(key)
        .ok_or_else(|| denied("Home idempotency key is invalid"))?;
    recorded_intent::validate(intent).map_err(denied)?;
    let AspectValue::UInt64(outcome) = field(&layout.outcome_identity_locator)? else {
        return Err(denied("Home idempotency outcome has foreign shape"));
    };
    if *outcome == 0 || *outcome == u64::MAX {
        return Err(denied("Home idempotency outcome identity is invalid"));
    }
    let AspectValue::UInt64(effects) = field(&layout.emitted_effect_count_locator)? else {
        return Err(denied("Home idempotency effect count has foreign shape"));
    };
    usize::try_from(*effects)
        .map_err(|_| denied("Home idempotency effect count exceeds this host"))?;
    native
        .history()
        .immutable_commit_receipt_for_version(row.created_at_version)
        .ok_or_else(|| denied("Home idempotency immutable creation receipt is missing"))?;
    Ok((*key.bytes(), *outcome))
}

fn validate_distinct(
    identities: &mut [([u8; 32], u64)],
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<(), Denial> {
    let work = identities
        .len()
        .checked_mul(identities.len().max(1).ilog2() as usize + 1)
        .and_then(|work| work.checked_mul(2))
        .and_then(|work| u64::try_from(work).ok())
        .ok_or_else(|| denied("Home idempotency ordering work overflow"))?;
    context
        .checkpoint(work)
        .map_err(|stop| request_read_denial::read(WorkCeilingDenial::Stopped(stop).into()))?;
    identities.sort_unstable_by_key(|(key, _)| *key);
    if identities.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(denied("Home idempotency contains duplicate keys"));
    }
    identities.sort_unstable_by_key(|(_, outcome)| *outcome);
    if identities.windows(2).any(|pair| pair[0].1 == pair[1].1) {
        return Err(denied(
            "Home idempotency contains duplicate outcome identities",
        ));
    }
    Ok(())
}

fn restore_frontier(
    identities: Vec<([u8; 32], u64)>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<(), Denial> {
    for (_, outcome) in identities {
        context
            .checkpoint(1)
            .map_err(|stop| request_read_denial::read(WorkCeilingDenial::Stopped(stop).into()))?;
        WorthQueryApplicationCommitOutcomeIdentity::restore(outcome)
            .ok_or_else(|| denied("Home idempotency outcome frontier is exhausted"))?;
    }
    Ok(())
}
