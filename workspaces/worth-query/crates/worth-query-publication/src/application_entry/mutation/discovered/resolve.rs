use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationStructuredValueBinding,
};
use worth_query_execution::facade::application_contribution::{
    WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationDiscoveredOutputConnection,
    WorthQueryApplicationProjection, WorthQueryPreparedRequiredOutputSource,
};

use super::{Discovery, RootConnection, WorthQueryDiscoveredProgramOutputHandle};
use crate::application_entry::demand::WorthQueryApplicationProgramDemandHandle;
use crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory;
use crate::application_entry::{
    WorthQueryApplicationReadObservation, WorthQueryApplicationRequest,
    WorthQueryOutputDemandControls, WorthQueryRequiredOutputPreparationDenial,
};

type DiscoveryBinding<Schema, Root> =
    <Discovery<Schema, Root> as ApplicationQueryIntent<Schema>>::Binding;
type DiscoveryQuery<Schema, Root> =
    <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::Query;
type DiscoveryValue<Schema, Root> = <<DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<
    Schema,
>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type RootDemand<Schema, Root> =
    <RootConnection<Schema, Root> as WorthQueryApplicationDiscoveredOutputConnection<Schema>>::Demand;
type RootFamily<Schema, Root> =
    <RootDemand<Schema, Root> as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type RootSource<Schema, Root> =
    <RootFamily<Schema, Root> as WorthQueryProducerOutputFamily<Schema>>::Source;
type RootSourceQuery<Schema, Root> =
    <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Query;
type RootSourceValue<Schema, Root> = <<RootSource<Schema, Root> as ApplicationQueryBinding<
    Schema,
>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

#[derive(Clone, Copy)]
pub(super) enum DiscoveredRootStartKind {
    Performed,
    Recovery,
}

pub(super) fn start_discovered_roots<'application, Schema, Program, Root>(
    application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    request: &WorthQueryApplicationRequest<'application, '_, '_, Schema>,
    receipt: &WorthQueryApplicationCommitReceipt,
    discovery: Discovery<Schema, Root>,
    retained: WorthQueryApplicationReadObservation,
    prepared: &WorthQueryPreparedRequiredOutputSource,
    controls: WorthQueryOutputDemandControls,
    start_kind: DiscoveredRootStartKind,
) -> Result<
    WorthQueryDiscoveredProgramOutputHandle<'application, Schema, Program, Root>,
    WorthQueryRequiredOutputPreparationDenial,
>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
    Root::Dependents:
        ProgramOutputContinuationFactory<'application, Schema, Program, RootDemand<Schema, Root>>,
    RootDemand<Schema, Root>: Clone,
    DiscoveryValue<Schema, Root>:
        WorthQueryApplicationProjection<Schema, DiscoveryQuery<Schema, Root>> + Clone,
    <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    RootSourceValue<Schema, Root>:
        WorthQueryApplicationProjection<Schema, RootSourceQuery<Schema, Root>> + Clone,
    <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = RootSource<Schema, Root>>,
    <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    let discovered = request
        .at(&retained)
        .query(discovery)
        .execute()
        .map_err(WorthQueryRequiredOutputPreparationDenial::SourceQuery)?;
    let row = discovered
        .rows()
        .first()
        .filter(|_| discovered.rows().len() == 1)
        .ok_or(WorthQueryRequiredOutputPreparationDenial::MissingSource)?;
    let demands = RootConnection::<Schema, Root>::demands_from_discovery(row)
        .map_err(WorthQueryRequiredOutputPreparationDenial::Connection)?;
    let mut sources = Vec::with_capacity(demands.len());
    let mut current_sources = Vec::with_capacity(demands.len());
    let mut stale_roots = Vec::new();
    for (index, demand) in demands.iter().enumerate() {
        let result = request
            .at(&retained)
            .query(demand.source_intent())
            .execute()
            .map_err(WorthQueryRequiredOutputPreparationDenial::SourceQuery)?;
        if result.rows().len() != 1 || result.observed_sources().len() != 1 {
            return Err(WorthQueryRequiredOutputPreparationDenial::MissingSource);
        }
        let retained_source = result.into_output_demand_source();
        let current_source = if matches!(start_kind, DiscoveredRootStartKind::Recovery) {
            let current = request
                .query(demand.source_intent())
                .execute()
                .map_err(WorthQueryRequiredOutputPreparationDenial::SourceQuery)?;
            let current_source = current.into_output_demand_source();
            match application
                .validate_recovered_discovered_program_root_currentness::<Root>(
                    &worth_query_execution::publication_boundary::program_publication_access(),
                    prepared,
                    &retained_source,
                    &current_source,
                ) {
                Ok(()) => {}
                Err(denial)
                    if denial.kind()
                        == worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::Superseded =>
                {
                    stale_roots.push(index);
                }
                Err(denial) => {
                    return Err(WorthQueryRequiredOutputPreparationDenial::DemandExecution(denial));
                }
            }
            Some(current_source)
        } else {
            None
        };
        sources.push(retained_source);
        current_sources.push(current_source);
    }
    application
        .bind_prepared_discovered_program_root_sources::<Root>(
            &worth_query_execution::publication_boundary::program_publication_access(),
            prepared,
            &sources,
        )
        .map_err(WorthQueryRequiredOutputPreparationDenial::DemandExecution)?;
    let mut roots = Vec::with_capacity(demands.len());
    let mut superseded = Vec::new();
    for (index, ((demand, source), current_source)) in demands
        .into_iter()
        .zip(sources)
        .zip(current_sources)
        .enumerate()
    {
        if stale_roots.contains(&index) {
            superseded.push(demand);
            continue;
        }
        let admitted = match start_kind {
            DiscoveredRootStartKind::Performed => application
                .admit_performed_discovered_program_root_output::<Root>(
                    &worth_query_execution::publication_boundary::program_publication_access(),
                    source,
                    controls.maximum_work().get(),
                    controls.maximum_retained_bytes().get(),
                    prepared,
                ),
            DiscoveredRootStartKind::Recovery => {
                let current_source = current_source
                    .ok_or(WorthQueryRequiredOutputPreparationDenial::MissingSource)?;
                application.recover_discovered_program_root_output::<Root>(
                    &worth_query_execution::publication_boundary::program_publication_access(),
                    source,
                    current_source,
                    controls.maximum_work().get(),
                    controls.maximum_retained_bytes().get(),
                    receipt,
                )
            }
        };
        let admitted = match admitted {
            Ok(admitted) => admitted,
            Err(denial)
                if denial.kind()
                    == worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::Superseded =>
            {
                superseded.push(demand);
                continue;
            }
            Err(denial) => {
                return Err(WorthQueryRequiredOutputPreparationDenial::DemandExecution(denial));
            }
        };
        roots.push(WorthQueryApplicationProgramDemandHandle::new(
            application,
            admitted,
            demand,
        ));
    }
    let source_lease = application
        .retain_discovered_program_source::<Root>(
            &worth_query_execution::publication_boundary::program_publication_access(),
            prepared,
        )
        .map_err(WorthQueryRequiredOutputPreparationDenial::DemandExecution)?;
    Ok(WorthQueryDiscoveredProgramOutputHandle::new(
        application,
        receipt.clone(),
        roots,
        superseded,
        source_lease,
        retained,
        controls,
    ))
}
