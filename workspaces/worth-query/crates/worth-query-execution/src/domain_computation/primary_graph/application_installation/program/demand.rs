use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationStructuredValueBinding,
};

use super::WorthQueryProgramApplicationRuntime;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedOutputDemand, WorthQueryApplicationDependentOutputConnection,
    WorthQueryApplicationOutputDemand, WorthQueryApplicationOutputDemandDisclosure,
    WorthQueryApplicationOutputDemandSource, WorthQueryApplicationRequiredOutputConnection,
    WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandNotifications, WorthQueryOutputDemandSettlement,
    WorthQueryPreparedRequiredOutputSource, WorthQueryProducerOutputFamily,
};

type RootDemand<Schema, Program> =
    <<Program as ApplicationProgramDefinition<Schema>>::Connections as WorthQueryApplicationRequiredOutputConnection<
        Schema,
    >>::Demand;
type RootFamily<Schema, Program> =
    <RootDemand<Schema, Program> as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type DependentDemand<Schema, Program> =
    <<Program as ApplicationProgramDefinition<Schema>>::DependentConnection as WorthQueryApplicationDependentOutputConnection<
        Schema,
    >>::Demand;
type DependentFamily<Schema, Program> =
    <DependentDemand<Schema, Program> as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type SourceQuery<Schema, Family> =
    <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<
        Schema,
    >>::Query;
type SourceValue<Schema, Family> =
    <<<Family as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<
        Schema,
    >>::ResultBinding as ApplicationStructuredValueBinding>::Value;

/// Admitted root output whose required custody came from this exact installed program.
pub struct WorthQueryAdmittedProgramRootOutput<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    admitted: WorthQueryAdmittedOutputDemand<Schema, RootFamily<Schema, Program>>,
}

/// Settled root output that alone permits this installed program's dependent admission.
///
/// An ordinary output settlement cannot substitute for the program-issued root phase.
///
/// ```compile_fail
/// use worth_query_declaration::facade::{
///     application_program::ApplicationProgramDefinition,
///     application_query::ApplicationQueryBinding,
///     application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
/// };
/// use worth_query_execution::facade::{
///     application_contribution::{WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily},
///     application_installation::WorthQueryProgramApplicationRuntime,
///     primary_graph::{WorthQueryApplicationDependentOutputConnection, WorthQueryApplicationOutputDemandSource, WorthQueryApplicationRequiredOutputConnection, WorthQueryOutputDemandSettlement},
/// };
/// type DepDemand<S, P> = <<P as ApplicationProgramDefinition<S>>::DependentConnection as WorthQueryApplicationDependentOutputConnection<S>>::Demand;
/// type Family<S, P> = <DepDemand<S, P> as WorthQueryApplicationOutputDemand<S>>::OutputFamily;
/// type Source<S, P> = <Family<S, P> as WorthQueryProducerOutputFamily<S>>::Source;
/// fn bypass<S, P>(
///     application: &WorthQueryProgramApplicationRuntime<S, P>,
///     ordinary: &WorthQueryOutputDemandSettlement,
///     source: WorthQueryApplicationOutputDemandSource<
///         <Source<S, P> as ApplicationQueryBinding<S>>::Query,
///         <<Source<S, P> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
///     >,
/// ) where
///     S: ApplicationSchema + 'static,
///     P: ApplicationProgramDefinition<S>,
///     P::Connections: WorthQueryApplicationRequiredOutputConnection<S>,
///     P::DependentConnection: WorthQueryApplicationDependentOutputConnection<S, RootDemand = <P::Connections as WorthQueryApplicationRequiredOutputConnection<S>>::Demand>,
/// {
///     let _ = application.admit_dependent_from_settled_program_root(ordinary, source, 1, 1);
/// }
/// ```
pub struct WorthQuerySettledProgramRootOutput<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    retained: std::sync::Arc<WorthQueryOutputDemandSettlement>,
    _program: std::marker::PhantomData<fn() -> (Schema, Program)>,
}

/// Admitted dependent output derived from this exact program root settlement.
pub struct WorthQueryAdmittedProgramDependentOutput<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Program::DependentConnection: WorthQueryApplicationDependentOutputConnection<Schema>,
{
    admitted: WorthQueryAdmittedOutputDemand<Schema, DependentFamily<Schema, Program>>,
}

pub enum WorthQueryProgramRootOutputAdvance<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    Pending,
    Settled(WorthQuerySettledProgramRootOutput<Schema, Program>),
}

impl<Schema, Program> WorthQueryAdmittedProgramRootOutput<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    pub fn observed_source(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryObservedSource<
        SourceQuery<Schema, RootFamily<Schema, Program>>,
    > {
        self.admitted.observed_source()
    }

    pub fn notifications(
        &self,
    ) -> Result<WorthQueryOutputDemandNotifications, WorthQueryOutputDemandDenial> {
        self.admitted.notifications()
    }

    pub fn close(&mut self) {
        self.admitted.close();
    }
}

impl<Schema, Program> WorthQueryAdmittedProgramDependentOutput<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Program::DependentConnection: WorthQueryApplicationDependentOutputConnection<Schema>,
{
    pub fn observed_source(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryObservedSource<
        SourceQuery<Schema, DependentFamily<Schema, Program>>,
    > {
        self.admitted.observed_source()
    }

    pub fn notifications(
        &self,
    ) -> Result<WorthQueryOutputDemandNotifications, WorthQueryOutputDemandDenial> {
        self.admitted.notifications()
    }

    pub fn close(&mut self) {
        self.admitted.close();
    }
}

impl<Schema, Program> WorthQuerySettledProgramRootOutput<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    pub fn retained(&self) -> std::sync::Arc<WorthQueryOutputDemandSettlement> {
        std::sync::Arc::clone(&self.retained)
    }
}

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Program::DependentConnection: WorthQueryApplicationDependentOutputConnection<
        Schema,
        RootDemand = RootDemand<Schema, Program>,
    >,
{
    pub fn recover_program_root_output(
        &self,
        source: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, RootFamily<Schema, Program>>,
            SourceValue<Schema, RootFamily<Schema, Program>>,
        >,
        maximum_work: usize,
        maximum_retained_bytes: usize,
        source_receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Result<WorthQueryAdmittedProgramRootOutput<Schema, Program>, WorthQueryOutputDemandDenial>
    {
        self.runtime
            .admit_recovered_output_demand::<RootFamily<Schema, Program>>(
                source,
                maximum_work,
                maximum_retained_bytes,
                source_receipt,
            )
            .map(|admitted| WorthQueryAdmittedProgramRootOutput { admitted })
    }

    pub fn admit_performed_program_root_output(
        &self,
        source: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, RootFamily<Schema, Program>>,
            SourceValue<Schema, RootFamily<Schema, Program>>,
        >,
        maximum_work: usize,
        maximum_retained_bytes: usize,
        prepared: &WorthQueryPreparedRequiredOutputSource,
    ) -> Result<WorthQueryAdmittedProgramRootOutput<Schema, Program>, WorthQueryOutputDemandDenial>
    {
        self.runtime
            .admit_performed_output_demand::<RootFamily<Schema, Program>>(
                source,
                maximum_work,
                maximum_retained_bytes,
                prepared,
            )
            .map(|admitted| WorthQueryAdmittedProgramRootOutput { admitted })
    }

    pub fn advance_program_root_output(
        &self,
        demand: &WorthQueryAdmittedProgramRootOutput<Schema, Program>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: WorthQueryApplicationOutputDemandDisclosure<
            SourceQuery<Schema, RootFamily<Schema, Program>>,
        >,
    ) -> Result<WorthQueryProgramRootOutputAdvance<Schema, Program>, WorthQueryOutputDemandDenial>
    where
        SourceValue<Schema, RootFamily<Schema, Program>>: 'static,
        SourceQuery<Schema, RootFamily<Schema, Program>>: 'static,
    {
        self.runtime
            .advance_output_demand(
                &demand.admitted,
                principal,
                request_scope,
                delivery_branch,
                disclosure,
            )
            .map(|progress| match progress {
                WorthQueryOutputDemandAdvance::Pending => {
                    WorthQueryProgramRootOutputAdvance::Pending
                }
                WorthQueryOutputDemandAdvance::Settled(retained) => {
                    WorthQueryProgramRootOutputAdvance::Settled(
                        WorthQuerySettledProgramRootOutput {
                            retained,
                            _program: std::marker::PhantomData,
                        },
                    )
                }
            })
    }

    pub fn admit_dependent_from_settled_program_root(
        &self,
        parent: &WorthQuerySettledProgramRootOutput<Schema, Program>,
        source: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, DependentFamily<Schema, Program>>,
            SourceValue<Schema, DependentFamily<Schema, Program>>,
        >,
        maximum_work: usize,
        maximum_retained_bytes: usize,
    ) -> Result<
        WorthQueryAdmittedProgramDependentOutput<Schema, Program>,
        WorthQueryOutputDemandDenial,
    > {
        if !parent.retained.belongs_to(&self.runtime) {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "dependent output parent belongs to another installed application",
            ));
        }
        let selected_source = source
            .observed_sources()
            .first()
            .filter(|_| source.rows().len() == 1 && source.observed_sources().len() == 1);
        let parent_commit = parent
            .retained
            .receipt()
            .committed_product_publication()
            .composite_commit();
        let parent_occurrence = parent.retained.receipt().product_branch().occurrence();
        if selected_source.is_none_or(|observed| {
            observed.selected_product_commit() != Some(parent_commit)
                || observed.selected_product_occurrence() != Some(parent_occurrence)
        }) {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "dependent output source was not discovered at its settled program root",
            ));
        }
        self.runtime
            .admit_required_output_demand::<DependentFamily<Schema, Program>>(
                source,
                maximum_work,
                maximum_retained_bytes,
            )
            .map(|admitted| WorthQueryAdmittedProgramDependentOutput { admitted })
    }

    pub fn advance_program_dependent_output(
        &self,
        demand: &WorthQueryAdmittedProgramDependentOutput<Schema, Program>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: WorthQueryApplicationOutputDemandDisclosure<
            SourceQuery<Schema, DependentFamily<Schema, Program>>,
        >,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        SourceValue<Schema, DependentFamily<Schema, Program>>: 'static,
        SourceQuery<Schema, DependentFamily<Schema, Program>>: 'static,
    {
        self.runtime.advance_output_demand(
            &demand.admitted,
            principal,
            request_scope,
            delivery_branch,
            disclosure,
        )
    }
}
