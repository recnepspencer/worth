use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition,
};
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

type Family<Schema, Demand> = <Demand as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type Source<Schema, Demand> =
    <Family<Schema, Demand> as WorthQueryProducerOutputFamily<Schema>>::Source;
type SourceQuery<Schema, Demand> =
    <Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::Query;
type SourceValue<Schema, Demand> =
    <<Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type RootConnectionRef<Schema, Root> =
    <Root as ApplicationOutputGraphShape<Schema>>::RootConnection;
type RootConnection<Schema, Root> =
    <RootConnectionRef<Schema, Root> as ApplicationConnectionShape<Schema>>::Binding;
pub type WorthQueryProgramRootDemand<Schema, Root> =
    <RootConnection<Schema, Root> as WorthQueryApplicationRequiredOutputConnection<Schema>>::Demand;
type ConnectionBinding<Schema, Connection> =
    <Connection as ApplicationConnectionShape<Schema>>::Binding;
type ConnectionDemand<Schema, Connection> =
    <ConnectionBinding<Schema, Connection> as WorthQueryApplicationDependentOutputConnection<
        Schema,
    >>::Demand;

/// Program-affine admission for one required output at any graph depth.
pub struct WorthQueryAdmittedProgramOutput<Schema, Program, Demand>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    admitted: WorthQueryAdmittedOutputDemand<Schema, Family<Schema, Demand>>,
    marker: std::marker::PhantomData<fn() -> Program>,
}

/// Settled output that authorizes admission of its declared child edges.
pub struct WorthQuerySettledProgramOutput<Schema, Program, Demand>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    retained: std::sync::Arc<WorthQueryOutputDemandSettlement>,
    marker: std::marker::PhantomData<fn() -> (Schema, Program, Demand)>,
}

pub enum WorthQueryProgramOutputAdvance<Schema, Program, Demand>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    Pending,
    Settled(WorthQuerySettledProgramOutput<Schema, Program, Demand>),
}

impl<Schema, Program, Demand> WorthQueryAdmittedProgramOutput<Schema, Program, Demand>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    pub fn observed_source(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryObservedSource<
        SourceQuery<Schema, Demand>,
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

impl<Schema, Program, Demand> WorthQuerySettledProgramOutput<Schema, Program, Demand>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    pub fn retained(&self) -> std::sync::Arc<WorthQueryOutputDemandSettlement> {
        std::sync::Arc::clone(&self.retained)
    }
}

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub fn recover_program_root_output<Root>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        source: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
            SourceValue<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
        >,
        maximum_work: usize,
        maximum_retained_bytes: usize,
        source_receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Result<
        WorthQueryAdmittedProgramOutput<Schema, Program, WorthQueryProgramRootDemand<Schema, Root>>,
        WorthQueryOutputDemandDenial,
    >
    where
        Root: ApplicationOutputGraphShape<Schema>,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    {
        if !self.contains_output_root::<Root>() {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignDemand,
                "selected output root is not installed for this program",
            ));
        }
        self.runtime
            .admit_recovered_output_demand::<
                Family<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
            >(
                source,
                maximum_work,
                maximum_retained_bytes,
                source_receipt,
            )
            .map(|admitted| WorthQueryAdmittedProgramOutput {
                admitted,
                marker: std::marker::PhantomData,
            })
    }

    pub fn admit_performed_program_root_output<Root>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        source: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
            SourceValue<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
        >,
        maximum_work: usize,
        maximum_retained_bytes: usize,
        prepared: &WorthQueryPreparedRequiredOutputSource,
    ) -> Result<
        WorthQueryAdmittedProgramOutput<Schema, Program, WorthQueryProgramRootDemand<Schema, Root>>,
        WorthQueryOutputDemandDenial,
    >
    where
        Root: ApplicationOutputGraphShape<Schema>,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    {
        if !self.contains_output_root::<Root>() {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignDemand,
                "selected output root is not installed for this program",
            ));
        }
        self.runtime
            .admit_performed_output_demand::<
                Family<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
            >(source, maximum_work, maximum_retained_bytes, prepared)
            .map(|admitted| WorthQueryAdmittedProgramOutput {
                admitted,
                marker: std::marker::PhantomData,
            })
    }
}

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub fn advance_program_output<Demand>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        demand: &WorthQueryAdmittedProgramOutput<Schema, Program, Demand>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: WorthQueryApplicationOutputDemandDisclosure<SourceQuery<Schema, Demand>>,
    ) -> Result<WorthQueryProgramOutputAdvance<Schema, Program, Demand>, WorthQueryOutputDemandDenial>
    where
        Demand: WorthQueryApplicationOutputDemand<Schema>,
        SourceValue<Schema, Demand>: 'static,
        SourceQuery<Schema, Demand>: 'static,
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
                WorthQueryOutputDemandAdvance::Pending => WorthQueryProgramOutputAdvance::Pending,
                WorthQueryOutputDemandAdvance::Settled(retained) => {
                    WorthQueryProgramOutputAdvance::Settled(WorthQuerySettledProgramOutput {
                        retained,
                        marker: std::marker::PhantomData,
                    })
                }
            })
    }

    pub fn admit_program_dependent_output<ParentDemand, Connection>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        parent: &WorthQuerySettledProgramOutput<Schema, Program, ParentDemand>,
        source: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, ConnectionDemand<Schema, Connection>>,
            SourceValue<Schema, ConnectionDemand<Schema, Connection>>,
        >,
        maximum_work: usize,
        maximum_retained_bytes: usize,
    ) -> Result<
        WorthQueryAdmittedProgramOutput<Schema, Program, ConnectionDemand<Schema, Connection>>,
        WorthQueryOutputDemandDenial,
    >
    where
        ParentDemand: WorthQueryApplicationOutputDemand<Schema>,
        Connection: ApplicationConnectionShape<Schema>,
        ConnectionBinding<Schema, Connection>:
            WorthQueryApplicationDependentOutputConnection<Schema, RootDemand = ParentDemand>,
    {
        if !self.contains_connection_type::<Connection>() {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignDemand,
                "dependent output connection is not installed for this program",
            ));
        }
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
                "dependent output source was not discovered at its settled program parent",
            ));
        }
        self.runtime
            .admit_required_output_demand::<Family<Schema, ConnectionDemand<Schema, Connection>>>(
                source,
                maximum_work,
                maximum_retained_bytes,
            )
            .map(|admitted| WorthQueryAdmittedProgramOutput {
                admitted,
                marker: std::marker::PhantomData,
            })
    }
}
