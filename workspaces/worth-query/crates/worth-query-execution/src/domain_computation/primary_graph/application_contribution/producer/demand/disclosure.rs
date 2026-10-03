use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_installation::facade::ApplicationSchema;

use super::super::InstalledProducerEdition;
use super::{
    FamilySourceQuery, FamilySourceValue, WorthQueryAdmittedOutputDemand,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind, WorthQueryProducerOutputFamily,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    product_operation::SharedSelectedProductOperation, WorthQueryApplicationBasisSelectionIdentity,
    WorthQueryApplicationOutputDemandSource, WorthQueryObservedSource,
    WorthQueryPrimaryGraphApplicationRuntime,
};

mod validated;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use validated::{
    FreshDisclosureAdmissionStop, FreshOutputDisclosure, ValidatedOutputDisclosure,
    ValidatedProducerInput,
};
use validated::{AdmittedDisclosure, RetainedProgramOutputDisclosure};

pub(super) fn validate_disclosure<Schema, Family>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    demand: &WorthQueryAdmittedOutputDemand<Schema, Family>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request_scope: &WorthQueryRequestScope,
    delivery_branch: crate::basis::WorthQueryProductBranch,
    allow_retained_program_basis: bool,
    source: WorthQueryApplicationOutputDemandSource<
        FamilySourceQuery<Schema, Family>,
        FamilySourceValue<Schema, Family>,
    >,
    edition: InstalledProducerEdition,
) -> Result<
    ValidatedOutputDisclosure<FamilySourceQuery<Schema, Family>, FamilySourceValue<Schema, Family>>,
    WorthQueryOutputDemandDenial,
>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    validate_source(
        runtime,
        &demand.observed_source,
        principal,
        request_scope,
        delivery_branch,
        allow_retained_program_basis,
        source,
        edition,
        DisclosureSelection::Single,
        Family::IDENTITY,
    )
}

/// This transition accepts only the actual inseparable query result. The
/// selected row is chosen under the query's original request-affinity proof.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn validate_readmitted<
    Schema,
    Query,
    Value,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    retained: &WorthQueryObservedSource<Query>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    branch: crate::basis::WorthQueryProductBranch,
    source: WorthQueryApplicationOutputDemandSource<Query, Value>,
    edition: InstalledProducerEdition,
    admission: &mut InvalidationEditAdmission,
    subject: &'static str,
) -> Result<FreshOutputDisclosure<Query, Value>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    match validate_source(
        runtime,
        retained,
        principal,
        request,
        branch,
        false,
        source,
        edition,
        DisclosureSelection::RetainedRoot(admission),
        subject,
    )? {
        ValidatedOutputDisclosure::Fresh(proof) => Ok(proof),
        ValidatedOutputDisclosure::RetainedProgram(_) => {
            unreachable!("exact selected basis was required")
        }
    }
}

/// Validate a fresh disclosure against the exact Product already selected for
/// a required wave. The ordinary and selected routes share the same source,
/// request-affinity, edition, and observation acceptance core.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn validate_readmitted_on_selected<
    Schema,
    Query,
    Value,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    retained: &WorthQueryObservedSource<Query>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    branch: crate::basis::WorthQueryProductBranch,
    shared: &SharedSelectedProductOperation<'_, Schema>,
    source: WorthQueryApplicationOutputDemandSource<Query, Value>,
    edition: InstalledProducerEdition,
    admission: &mut InvalidationEditAdmission,
    subject: &'static str,
) -> Result<FreshOutputDisclosure<Query, Value>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    admission.charge_external_work(2).map_err(|_| {
        denial(
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
            subject,
        )
    })?;
    if !std::ptr::eq(runtime, shared.selected().application())
        || shared
            .selected()
            .product()
            .read_lease_ref()
            .observation()
            .lifecycle_incarnation()
            != branch.occurrence()
    {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::ForeignDemand,
            subject,
        ));
    }
    match validate_source(
        runtime,
        retained,
        principal,
        request,
        branch,
        false,
        source,
        edition,
        DisclosureSelection::RetainedSelected(
            admission,
            shared.selected().product().read_lease_ref().observation(),
        ),
        subject,
    )? {
        ValidatedOutputDisclosure::Fresh(proof) => Ok(proof),
        ValidatedOutputDisclosure::RetainedProgram(_) => {
            unreachable!("exact selected basis was required")
        }
    }
}

enum DisclosureSelection<'a> {
    Single,
    RetainedRoot(&'a mut InvalidationEditAdmission),
    RetainedSelected(
        &'a mut InvalidationEditAdmission,
        &'a worth_runtime_world::facade::ProductBranchObservation,
    ),
}

fn validate_source<Schema, Query, Value>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    retained: &WorthQueryObservedSource<Query>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    branch: crate::basis::WorthQueryProductBranch,
    allow_retained_program_basis: bool,
    source: WorthQueryApplicationOutputDemandSource<Query, Value>,
    edition: InstalledProducerEdition,
    mut selection: DisclosureSelection<'_>,
    subject: &'static str,
) -> Result<ValidatedOutputDisclosure<Query, Value>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    let (rows, disclosure) = source.into_parts();
    let (sources, affinity, receipt) = disclosure.into_parts();
    if let (Some(affinity), DisclosureSelection::RetainedSelected(admission, _)) =
        (affinity.as_ref(), &mut selection)
    {
        let metadata = affinity
            .comparison_metadata_visits(principal)
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                    subject,
                )
            })?;
        admission.charge_external_work(metadata).map_err(|_| {
            denial(
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                subject,
            )
        })?;
        let comparison = affinity.comparison_work(principal).ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                subject,
            )
        })?;
        admission.charge_external_work(comparison).map_err(|_| {
            denial(
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                subject,
            )
        })?;
    }
    let affinity = affinity
        .filter(|affinity| affinity.admits(principal, request))
        .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::Superseded, subject))?;
    if rows.len() != sources.len()
        || (matches!(selection, DisclosureSelection::Single) && rows.len() != 1)
    {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::Superseded,
            subject,
        ));
    }
    let mut found = None;
    for (value, source) in rows.into_iter().zip(sources) {
        if let DisclosureSelection::RetainedRoot(admission)
        | DisclosureSelection::RetainedSelected(admission, _) = &mut selection
        {
            admission.charge_external_work(4).map_err(|_| {
                denial(
                    WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                    subject,
                )
            })?;
        }
        if matches!(selection, DisclosureSelection::Single)
            || source.source_root() == retained.source_root()
        {
            if found.replace((value, source)).is_some() {
                return Err(denial(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    subject,
                ));
            }
        }
    }
    let (value, source) =
        found.ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::Superseded, subject))?;
    let selected = match &selection {
        DisclosureSelection::RetainedSelected(_, _) => None,
        DisclosureSelection::Single | DisclosureSelection::RetainedRoot(_) => {
            Some(runtime.on_branch(branch).select().map_err(|error| {
                WorthQueryOutputDemandDenial::product_selection(
                    error,
                    format!("{subject}: disclosure product selection"),
                )
            })?)
        }
    };
    let selected_current = match &selection {
        DisclosureSelection::RetainedSelected(_, current) => Some(*current),
        DisclosureSelection::Single | DisclosureSelection::RetainedRoot(_) => None,
    };
    let current = selected_current.unwrap_or_else(|| {
        selected
            .as_ref()
            .expect("ordinary disclosure selected its Product")
            .product()
            .observation()
    });
    let WorthQueryApplicationBasisSelectionIdentity::Product(disclosed) = &source.selection else {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::Superseded,
            subject,
        ));
    };
    let WorthQueryApplicationBasisSelectionIdentity::Product(original) = &retained.selection else {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::ForeignDemand,
            subject,
        ));
    };
    if let DisclosureSelection::RetainedSelected(admission, _) = &mut selection {
        // Metadata reads precede the variable-width source comparisons. The
        // same exact observation is then checked by the original validator.
        admission.charge_external_work(10).map_err(|_| {
            denial(
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                subject,
            )
        })?;
        let disclosed_name = disclosed.branch_identity().name().as_str().len();
        let current_name = current.branch_identity().name().as_str().len();
        let original_name = original.branch_identity().name().as_str().len();
        let comparison = disclosed_name
            .checked_mul(3)
            .and_then(|work| work.checked_add(current_name.checked_mul(2)?))
            .and_then(|work| work.checked_add(original_name))
            .and_then(|work| work.checked_add(source.query_identifier.len()))
            .and_then(|work| work.checked_add(retained.query_identifier.len()))
            // Two schema digests and two Query-identity comparisons read 128
            // fixed bytes; the remaining 32 visits cover source, receipt,
            // Product generation/commit/basis and root axes.
            .and_then(|work| work.checked_add(160))
            .and_then(|work| u64::try_from(work).ok())
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                    subject,
                )
            })?;
        admission.charge_external_work(comparison).map_err(|_| {
            denial(
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                subject,
            )
        })?;
    }
    if source.runtime_authority != runtime.runtime.authority_identity().as_u64()
        || source.schema_binding != runtime.installed_schema.binding_identity()
        || receipt.query_identity() != &source.query_identity
        || source.query_identity != retained.query_identity
        || source.query_identifier != retained.query_identifier
        || (!allow_retained_program_basis && !disclosed.matches_observation(current))
        || (allow_retained_program_basis && !disclosed.same_branch_occurrence_observation(current))
        || !original.same_branch_occurrence(disclosed)
        || source.model_root != retained.model_root
        || source.source_root() != retained.source_root()
    {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::Superseded,
            subject,
        ));
    }
    let fresh = disclosed.matches_observation(current);
    let admitted = AdmittedDisclosure {
        value,
        source,
        affinity,
        edition,
    };
    Ok(if fresh {
        ValidatedOutputDisclosure::Fresh(FreshOutputDisclosure(admitted))
    } else {
        ValidatedOutputDisclosure::RetainedProgram(RetainedProgramOutputDisclosure(admitted))
    })
}

fn denial(
    kind: WorthQueryOutputDemandDenialKind,
    subject: impl Into<std::borrow::Cow<'static, str>>,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, subject)
}
