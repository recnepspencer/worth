use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;

use super::super::InstalledProducerEdition;
use super::FamilySourceQuery;
use super::FamilySourceValue;
use super::WorthQueryAdmittedOutputDemand;
use super::WorthQueryOutputDemandDenial;
use super::WorthQueryOutputDemandDenialKind;
use super::WorthQueryProducerOutputFamily;

use crate::domain_computation::primary_graph::product_operation::SharedSelectedProductOperation;
use crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity;
use crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemandSource;
use crate::domain_computation::primary_graph::WorthQueryObservedSource;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

mod producer_input;
mod required_successor;
mod validated;
pub(super) use required_successor::validate_required_successor;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use producer_input::ValidatedProducerInput;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use validated::FreshOutputDisclosure;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use validated::ValidatedOutputDisclosure;
use validated::AdmittedDisclosure;
use validated::RetainedProgramOutputDisclosure;

pub(super) fn validate_disclosure<Schema, Family>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        execution,
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    retained: &WorthQueryObservedSource<Query>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    branch: crate::basis::WorthQueryProductBranch,
    source: WorthQueryApplicationOutputDemandSource<Query, Value>,
    edition: InstalledProducerEdition,

    subject: &'static str,
) -> Result<FreshOutputDisclosure<Query, Value>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    match validate_source(
        execution,
        runtime,
        retained,
        principal,
        request,
        branch,
        false,
        source,
        edition,
        DisclosureSelection::RetainedRoot,
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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    retained: &WorthQueryObservedSource<Query>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    branch: crate::basis::WorthQueryProductBranch,
    shared: &SharedSelectedProductOperation<'_, Schema>,
    source: WorthQueryApplicationOutputDemandSource<Query, Value>,
    edition: InstalledProducerEdition,

    subject: &'static str,
) -> Result<FreshOutputDisclosure<Query, Value>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
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
        execution,
        runtime,
        retained,
        principal,
        request,
        branch,
        false,
        source,
        edition,
        DisclosureSelection::RetainedSelected(
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
    RetainedRoot,
    RetainedSelected(&'a worth_runtime_world::facade::ProductBranchObservation),
}

fn validate_source<Schema, Query, Value>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        if (matches!(selection, DisclosureSelection::Single)
            || source.source_root() == retained.source_root())
            && found.replace((value, source)).is_some()
        {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::Superseded,
                subject,
            ));
        }
    }
    let (value, source) =
        found.ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::Superseded, subject))?;
    let selected = match &selection {
        DisclosureSelection::RetainedSelected(_) => None,
        DisclosureSelection::Single | DisclosureSelection::RetainedRoot => Some(
            runtime
                .on_branch(branch)
                .select_with_request(execution)
                .map_err(|error| {
                    WorthQueryOutputDemandDenial::product_selection(
                        error,
                        format!("{subject}: disclosure product selection"),
                    )
                })?,
        ),
    };
    let selected_current = match &selection {
        DisclosureSelection::RetainedSelected(current) => Some(*current),
        DisclosureSelection::Single | DisclosureSelection::RetainedRoot => None,
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
    if let DisclosureSelection::RetainedSelected(_) = &mut selection {
        // Metadata reads precede the variable-width source comparisons. The
        // same exact observation is then checked by the original validator.
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
        admission_edition: edition,
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
