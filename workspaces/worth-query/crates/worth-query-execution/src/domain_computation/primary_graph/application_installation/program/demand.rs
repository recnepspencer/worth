use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationProgramInventoryIdentity,
};
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationStructuredValueBinding,
};

use super::WorthQueryProgramExecutionPort;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedOutputDemand, WorthQueryApplicationOutputDemand,
    WorthQueryApplicationOutputDemandDisclosure, WorthQueryApplicationOutputDemandSource,
    WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryOutputDemandNotifications, WorthQueryOutputDemandSettlement,
    WorthQueryPreparedRequiredOutputSource, WorthQueryProducerOutputFamily,
};

type Family<Schema, Demand> = <Demand as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type SourceQuery<Schema, Demand> = <<Family<Schema, Demand> as WorthQueryProducerOutputFamily<
    Schema,
>>::Source as ApplicationQueryBinding<Schema>>::Query;
type SourceValue<Schema, Demand> =
    <<<Family<Schema, Demand> as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

/// One admitted node in an installed program plan.
pub struct WorthQueryAdmittedProgramOutput<Schema, Program, Inventory, Demand>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    admitted: WorthQueryAdmittedOutputDemand<Schema, Family<Schema, Demand>>,
    marker: PhantomData<fn() -> (Program, Inventory, Demand)>,
}

/// Settlement authority sealed to one installed program and requested inventory.
pub struct WorthQuerySettledProgramOutput<Schema, Program, Inventory, Demand>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    retained: Arc<WorthQueryOutputDemandSettlement>,
    marker: PhantomData<fn() -> (Schema, Program, Inventory, Demand)>,
}

pub enum WorthQueryProgramOutputAdvance<Schema, Program, Inventory, Demand>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    Pending,
    Settled(WorthQuerySettledProgramOutput<Schema, Program, Inventory, Demand>),
}

impl<Schema, Program, Inventory, Demand>
    WorthQueryAdmittedProgramOutput<Schema, Program, Inventory, Demand>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
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

impl<Schema, Program, Inventory, Demand>
    WorthQuerySettledProgramOutput<Schema, Program, Inventory, Demand>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    pub fn retained(&self) -> Arc<WorthQueryOutputDemandSettlement> {
        Arc::clone(&self.retained)
    }
}

impl<Schema, Program> WorthQueryProgramExecutionPort<'_, Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub fn recover_program_output<Inventory, Demand>(
        &self,
        source: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, Demand>,
            SourceValue<Schema, Demand>,
        >,
        maximum_work: usize,
        maximum_retained_bytes: usize,
        source_receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Result<
        WorthQueryAdmittedProgramOutput<Schema, Program, Inventory, Demand>,
        WorthQueryOutputDemandDenial,
    >
    where
        Inventory: ApplicationProgramInventoryIdentity,
        Demand: WorthQueryApplicationOutputDemand<Schema>,
    {
        self.application
            .runtime
            .admit_recovered_output_demand::<Family<Schema, Demand>>(
                source,
                maximum_work,
                maximum_retained_bytes,
                source_receipt,
            )
            .map(|admitted| WorthQueryAdmittedProgramOutput {
                admitted,
                marker: PhantomData,
            })
    }

    pub fn admit_performed_program_output<Inventory, Demand>(
        &self,
        source: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, Demand>,
            SourceValue<Schema, Demand>,
        >,
        maximum_work: usize,
        maximum_retained_bytes: usize,
        prepared: &WorthQueryPreparedRequiredOutputSource,
    ) -> Result<
        WorthQueryAdmittedProgramOutput<Schema, Program, Inventory, Demand>,
        WorthQueryOutputDemandDenial,
    >
    where
        Inventory: ApplicationProgramInventoryIdentity,
        Demand: WorthQueryApplicationOutputDemand<Schema>,
    {
        self.application
            .runtime
            .admit_performed_output_demand::<Family<Schema, Demand>>(
                source,
                maximum_work,
                maximum_retained_bytes,
                prepared,
            )
            .map(|admitted| WorthQueryAdmittedProgramOutput {
                admitted,
                marker: PhantomData,
            })
    }

    pub fn advance_program_output<Inventory, Demand>(
        &self,
        demand: &WorthQueryAdmittedProgramOutput<Schema, Program, Inventory, Demand>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: WorthQueryApplicationOutputDemandDisclosure<SourceQuery<Schema, Demand>>,
    ) -> Result<
        WorthQueryProgramOutputAdvance<Schema, Program, Inventory, Demand>,
        WorthQueryOutputDemandDenial,
    >
    where
        Inventory: ApplicationProgramInventoryIdentity,
        Demand: WorthQueryApplicationOutputDemand<Schema>,
        SourceValue<Schema, Demand>: 'static,
        SourceQuery<Schema, Demand>: 'static,
    {
        self.application
            .runtime
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
                        marker: PhantomData,
                    })
                }
            })
    }

    pub fn admit_dependent_program_output<Inventory, ParentDemand, Demand>(
        &self,
        parent: &WorthQuerySettledProgramOutput<Schema, Program, Inventory, ParentDemand>,
        source: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, Demand>,
            SourceValue<Schema, Demand>,
        >,
        maximum_work: usize,
        maximum_retained_bytes: usize,
    ) -> Result<
        WorthQueryAdmittedProgramOutput<Schema, Program, Inventory, Demand>,
        WorthQueryOutputDemandDenial,
    >
    where
        Inventory: ApplicationProgramInventoryIdentity,
        ParentDemand: WorthQueryApplicationOutputDemand<Schema>,
        Demand: WorthQueryApplicationOutputDemand<Schema>,
    {
        if !parent.retained.belongs_to(&self.application.runtime) {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "program parent belongs to another installed application",
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
                WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "dependent source was not discovered at its settled program parent",
            ));
        }
        self.application
            .runtime
            .admit_required_output_demand::<Family<Schema, Demand>>(
                source,
                maximum_work,
                maximum_retained_bytes,
            )
            .map(|admitted| WorthQueryAdmittedProgramOutput {
                admitted,
                marker: PhantomData,
            })
    }
}
