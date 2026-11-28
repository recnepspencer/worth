use super::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

pub fn execute_fresh<Schema, Binding>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &WorthQueryAdvancementPhase<'_>,
    installed: &TypedInstalledProducer<Schema, Binding>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external_principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request_scope: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    required_output: RequiredOutputDemandContext,
    provider: &std::sync::Arc<Binding::Provider>,
    disclosure: FreshOutputDisclosure<SourceQuery<Schema, Binding>, SourceValue<Schema, Binding>>,
    successor_of: Option<[u8; 32]>,
    commit_authority: worth_query_execution_lineage::facade::domain_computation::primary_graph::application_contribution::WorthQueryProducerCommitAuthority,
    edition: super::super::InstalledProducerEdition,
    producer_contacts: &mut usize,
) -> Result<ProducerExecutionOutcome, ProducerExecutionStop>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    SourceValue<Schema, Binding>:
        crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
            Schema,
            SourceQuery<Schema, Binding>,
        >,
{
    let selected = runtime
        .on_branch(branch)
        .select_with_request(execution)
        .map_err(|error| match error {
            stopped @ crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(_) => {
                WorthQueryOutputDemandDenial::product_selection(stopped, Binding::IDENTITY)
            }
            refused @ crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(_) => {
                failed(Binding::IDENTITY, refused)
            }
        })?;
    if !disclosure.admits(
        external_principal,
        request_scope,
        selected.product().observation(),
    ) {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::Superseded,
            Binding::IDENTITY,
        )
        .into());
    }
    let (source, observed_source) = disclosure.into_parts();
    let input = provider.operation_input(&source);
    let super::ordinary_authorization::OrdinaryProducerAuthorization {
        shared,
        principal,
        admission,
        scope,
    } = super::ordinary_authorization::prepare::<Schema, Binding>(
        execution,
        selected,
        installed,
        external_principal,
        request_scope,
        &input,
    )?;
    installed.refuse_uncaptured_production()?;
    post_authorization::execute_authorized::<Schema, Binding>(
        phase,
        runtime,
        &shared,
        external_principal,
        request_scope,
        &principal,
        Binding::PREDECESSOR_READS
            .or(Binding::ORIGINAL_READS)
            .as_ref()
            .map(|_| &scope),
        admission,
        required_output,
        provider,
        source,
        observed_source,
        input,
        None,
        successor_of,
        commit_authority,
        edition,
        producer_contacts,
    )
}
