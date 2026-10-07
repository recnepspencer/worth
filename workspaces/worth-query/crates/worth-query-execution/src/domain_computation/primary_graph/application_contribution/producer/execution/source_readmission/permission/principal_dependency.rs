//! Request-scoped currentness custody for a freshly resolved principal.

use super::*;
use crate::domain_computation::authorization::WorthQueryPrincipalCurrentnessDependency;
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;

pub(super) fn capture<Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedPrincipal<
        Schema,
        BoundPrincipal<Schema, Binding>,
        BoundPrincipalIdentity<Schema, Binding>,
    >,
    session: WorthQueryGraphWorkSessionIdentity,
    admission: &mut InvalidationEditAdmission,
) -> Result<WorthQueryPrincipalCurrentnessDependency, ProducerExecutionStop>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    let graph = runtime
        .runtime
        .primary_graph()
        .ok_or_else(|| unavailable::<Schema, Binding>())?;
    let binding = principal.binding();
    let lookup_work = graph
        .layout
        .principal_lookup_work(binding)
        .ok_or_else(|| work_denial::<Schema, Binding>())?;
    admission
        .charge_external_work(lookup_work)
        .map_err(|_| work_denial::<Schema, Binding>())?;
    let layout = graph
        .layout
        .principal_binding(binding)
        .ok_or_else(|| unavailable::<Schema, Binding>())?;
    let locators = [
        &layout.identity_locator,
        &layout.status_locator,
        &layout.principal_identity_locator,
    ];
    let field_visits = locators
        .iter()
        .try_fold(3usize, |sum, locator| {
            sum.checked_add(locator.field_path().fields().len())
        })
        .ok_or_else(|| work_denial::<Schema, Binding>())?;
    admission
        .charge_external_work(
            u64::try_from(field_visits).map_err(|_| work_denial::<Schema, Binding>())?,
        )
        .map_err(|_| work_denial::<Schema, Binding>())?;
    let (layout_work, layout_bytes) = locators
        .iter()
        .try_fold((0usize, 0usize), |(work, bytes), locator| {
            let work = work.checked_add(locator.aspect().aspect_key().as_str().len())?;
            let work = locator
                .field_path()
                .fields()
                .iter()
                .try_fold(work, |sum, field| sum.checked_add(field.as_str().len()))?;
            Some((
                work,
                bytes.checked_add(locator.owned_allocation_capacity_bytes())?,
            ))
        })
        .ok_or_else(|| work_denial::<Schema, Binding>())?;
    let (freshness_work, freshness_bytes) = principal
        .freshness()
        .clone_requirements()
        .ok_or_else(|| work_denial::<Schema, Binding>())?;
    let copy_work = u64::try_from(layout_work)
        .ok()
        .and_then(|work| work.checked_add(freshness_work))
        .and_then(|work| work.checked_add(u64::try_from(binding.len()).ok()?))
        .ok_or_else(|| work_denial::<Schema, Binding>())?;
    let binding_bytes = binding
        .len()
        .checked_add(2 * std::mem::size_of::<usize>())
        .and_then(|n| n.checked_add(7))
        .map(|n| n & !7)
        .ok_or_else(|| work_denial::<Schema, Binding>())?;
    let copy_bytes = u64::try_from(layout_bytes)
        .ok()
        .and_then(|bytes| bytes.checked_add(freshness_bytes))
        .and_then(|bytes| bytes.checked_add(u64::try_from(binding_bytes).ok()?))
        .ok_or_else(|| work_denial::<Schema, Binding>())?;
    admission
        .charge_external_work(copy_work)
        .map_err(|_| work_denial::<Schema, Binding>())?;
    admission
        .admit_read_scratch(copy_bytes)
        .map_err(|stop| resource_denial::<Schema, Binding>(stop))?;
    Ok(WorthQueryPrincipalCurrentnessDependency::capture(
        session, principal, layout,
    ))
}

fn unavailable<Schema, Binding>() -> ProducerExecutionStop
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    denial(
        WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
        Binding::IDENTITY,
    )
    .into()
}

fn work_denial<Schema, Binding>() -> ProducerExecutionStop
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    denial(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        Binding::IDENTITY,
    )
    .into()
}

fn resource_denial<Schema, Binding>(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> ProducerExecutionStop
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
    let kind = match stop {
        Stop::WorkExhausted { .. } | Stop::WorkCounterOverflow => {
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        }
        _ => WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
    };
    denial(kind, Binding::IDENTITY).into()
}
