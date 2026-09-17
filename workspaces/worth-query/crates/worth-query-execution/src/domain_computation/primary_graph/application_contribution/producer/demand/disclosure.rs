use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    FamilySourceQuery, WorthQueryAdmittedOutputDemand, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind, WorthQueryProducerOutputFamily,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationBasisSelectionIdentity, WorthQueryApplicationOutputDemandDisclosure,
    WorthQueryObservedSource, WorthQueryPrimaryGraphApplicationRuntime,
};
pub(super) fn validate_disclosure<Schema, Family>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    demand: &WorthQueryAdmittedOutputDemand<Schema, Family>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request_scope: &WorthQueryRequestScope,
    delivery_branch: crate::basis::WorthQueryProductBranch,
    disclosure: WorthQueryApplicationOutputDemandDisclosure<FamilySourceQuery<Schema, Family>>,
) -> Result<WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    let (mut sources, request_affinity, receipt) = disclosure.into_parts();
    if !request_affinity.is_some_and(|affinity| affinity.admits(principal, request_scope)) {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::Superseded,
            Family::IDENTITY,
        ));
    }
    let source = (sources.len() == 1)
        .then(|| sources.pop())
        .flatten()
        .ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::Superseded,
                Family::IDENTITY,
            )
        })?;
    let selected = runtime
        .on_branch(delivery_branch)
        .select()
        .map_err(|error| {
            denial(
                WorthQueryOutputDemandDenialKind::Superseded,
                format!("{}: {error:?}", Family::IDENTITY),
            )
        })?;
    let current = crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
        selected.product().observation(),
    );
    let WorthQueryApplicationBasisSelectionIdentity::Product(disclosed) = &source.selection else {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::Superseded,
            Family::IDENTITY,
        ));
    };
    let WorthQueryApplicationBasisSelectionIdentity::Product(original) =
        &demand.observed_source.selection
    else {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::ForeignDemand,
            Family::IDENTITY,
        ));
    };
    if source.runtime_authority != runtime.runtime.authority_identity().as_u64()
        || source.schema_binding != runtime.installed_schema.binding_identity()
        || receipt.query_identity() != &source.query_identity
        || source.query_identity != demand.observed_source.query_identity
        || source.query_identifier != demand.observed_source.query_identifier
        || disclosed != &current
        || !original.same_branch_occurrence(disclosed)
        || source.model_root != demand.observed_source.model_root
        || source.footprint.root != demand.observed_source.footprint.root
    {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::Superseded,
            Family::IDENTITY,
        ));
    }
    Ok(source)
}

fn denial(
    kind: WorthQueryOutputDemandDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, subject)
}
