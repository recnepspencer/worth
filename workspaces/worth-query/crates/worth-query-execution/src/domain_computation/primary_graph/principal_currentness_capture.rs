//! Retain principal currentness from the installed layout under one admission.

use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::{
    InvalidationEditAdmission, WorthQueryAuthenticatedPrincipal,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryPrincipalBindingLayout,
    WorthQueryPrincipalFreshnessEvidence,
};
use crate::domain_computation::authorization::WorthQueryPrincipalCurrentnessDependency;
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;

pub(in crate::domain_computation) enum PrincipalCurrentnessCaptureStop {
    MissingGraph,
    MissingBinding,
    Admission(CompanionPreflightStop),
    AccountingOverflow,
}

pub(in crate::domain_computation) fn capture_principal_currentness_admitted<
    Schema,
    Principal,
    PrincipalIdentity,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    session: WorthQueryGraphWorkSessionIdentity,
    admission: &mut InvalidationEditAdmission,
) -> Result<WorthQueryPrincipalCurrentnessDependency, PrincipalCurrentnessCaptureStop>
where
    Schema: ApplicationSchema,
{
    use PrincipalCurrentnessCaptureStop as Stop;
    // The selected binding header, runtime graph header and the base-six
    // lookup-bound loop all precede the variable comparison charge. Each
    // loop iteration grows the minimum key count, so usize::BITS is a finite
    // upper bound even on the widest installed catalog.
    admission
        .charge_external_work(u64::from(usize::BITS) + 6)
        .map_err(Stop::Admission)?;
    let binding = principal.binding();
    let subject_bytes = u64::try_from(binding.len()).map_err(|_| Stop::AccountingOverflow)?;
    // Stale graph/layout can return an owned binding subject. First refusal
    // remains allocation-free at the caller.
    admission
        .admit_read_scratch(subject_bytes)
        .and_then(|()| admission.charge_external_work(subject_bytes))
        .map_err(Stop::Admission)?;
    let graph = runtime.runtime.primary_graph().ok_or(Stop::MissingGraph)?;
    let lookup_work = graph
        .layout
        .principal_lookup_work(binding)
        .ok_or(Stop::AccountingOverflow)?;
    admission
        .charge_external_work(lookup_work)
        .map_err(Stop::Admission)?;
    let layout = graph
        .layout
        .principal_binding(binding)
        .ok_or(Stop::MissingBinding)?;
    admission.charge_external_work(6).map_err(Stop::Admission)?;
    let locators = [
        &layout.identity_locator,
        &layout.status_locator,
        &layout.principal_identity_locator,
    ];
    let field_visits = locators.iter().try_fold(3usize, |sum, locator| {
        sum.checked_add(locator.field_path().fields().len())
    });
    let field_visits = field_visits.ok_or(Stop::AccountingOverflow)?;
    let field_entries = field_visits - 3;
    // One pass reads initialized widths; owned_allocation_capacity_bytes
    // performs a second pass through the same selected field keys.
    let measurement_visits = field_visits
        .checked_add(field_entries)
        .ok_or(Stop::AccountingOverflow)?;
    admission
        .charge_external_work(
            u64::try_from(measurement_visits).map_err(|_| Stop::AccountingOverflow)?,
        )
        .map_err(Stop::Admission)?;
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
        .ok_or(Stop::AccountingOverflow)?;
    // The two retained scalar variants can each inspect two initialized
    // widths while their clone requirement is measured.
    admission.charge_external_work(4).map_err(Stop::Admission)?;
    let (freshness_work, freshness_bytes) = principal
        .freshness()
        .clone_requirements()
        .ok_or(Stop::AccountingOverflow)?;
    let field_slots = field_entries
        .checked_mul(std::mem::size_of::<worth_foundational::facade::FieldKey>())
        .ok_or(Stop::AccountingOverflow)?;
    let inline_work = std::mem::size_of::<WorthQueryPrimaryPrincipalBindingLayout>()
        .checked_add(std::mem::size_of::<WorthQueryPrincipalFreshnessEvidence>())
        .and_then(|n| {
            n.checked_add(std::mem::size_of::<WorthQueryPrincipalCurrentnessDependency>())
        })
        .and_then(|n| {
            n.checked_add(crate::domain_computation::arc_str_layout::initialized_header_work())
        })
        .ok_or(Stop::AccountingOverflow)?;
    let copy_work = u64::try_from(layout_work)
        .ok()
        .and_then(|work| work.checked_add(freshness_work))
        .and_then(|work| work.checked_add(u64::try_from(binding.len()).ok()?))
        .and_then(|work| work.checked_add(u64::try_from(field_slots).ok()?))
        .and_then(|work| work.checked_add(u64::try_from(inline_work).ok()?))
        .ok_or(Stop::AccountingOverflow)?;
    let binding_bytes = crate::domain_computation::arc_str_layout::backing_bytes(binding.len())
        .ok_or(Stop::AccountingOverflow)?;
    let copy_bytes = u64::try_from(layout_bytes)
        .ok()
        .and_then(|bytes| bytes.checked_add(freshness_bytes))
        .and_then(|bytes| bytes.checked_add(u64::try_from(binding_bytes).ok()?))
        .ok_or(Stop::AccountingOverflow)?;
    admission
        .charge_external_work(copy_work)
        .and_then(|()| admission.admit_read_scratch(copy_bytes))
        .map_err(Stop::Admission)?;
    Ok(WorthQueryPrincipalCurrentnessDependency::capture(
        session, principal, layout,
    ))
}
