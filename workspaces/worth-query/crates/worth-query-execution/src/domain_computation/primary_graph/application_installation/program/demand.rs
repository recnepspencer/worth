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
mod dependent;
mod discovered_roots;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedOutputDemand, WorthQueryApplicationDependentOutputConnection,
    WorthQueryApplicationOutputDemand, WorthQueryApplicationOutputDemandSource,
    WorthQueryApplicationRequiredOutputConnection, WorthQueryOutputDemandAdvance,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandNotifications,
    WorthQueryOutputDemandSettlement, WorthQueryPreparedRequiredOutputSource,
    WorthQueryProducerOutputFamily,
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
    target_feature: std::any::TypeId,
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
    target_feature: std::any::TypeId,
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
    pub fn admit_program_root_output<Root>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        source: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
            SourceValue<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
        >,
        maximum_work: usize,
        maximum_retained_bytes: usize,
    ) -> Result<
        WorthQueryAdmittedProgramOutput<Schema, Program, WorthQueryProgramRootDemand<Schema, Root>>,
        WorthQueryOutputDemandDenial,
    >
    where
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    {
        if !self.contains_output_root::<Root>() {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignDemand,
                "selected output root is not installed for this program",
            ));
        }
        self.validate_root_artifact_demand::<Root>(maximum_work, maximum_retained_bytes)?;
        self.runtime
            .admit_required_output_demand::<
                Family<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
            >(source, maximum_work, maximum_retained_bytes)
            .map(|admitted| WorthQueryAdmittedProgramOutput {
                admitted,
                target_feature: std::any::TypeId::of::<
                    <RootConnectionRef<Schema, Root> as ApplicationConnectionShape<Schema>>::TargetFeature,
                >(),
                marker: std::marker::PhantomData,
            })
    }

    pub fn recover_prepared_program_root_source<Root>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Result<
        (
            std::sync::Arc<
                crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
            >,
            WorthQueryPreparedRequiredOutputSource,
        ),
        WorthQueryOutputDemandDenial,
    >
    where
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    {
        if !self.contains_output_root::<Root>() {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignDemand,
                "selected output root is not installed for this program",
            ));
        }
        self.runtime.validate_recovered_output_root_kind(
            receipt,
            crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind::Required(std::any::TypeId::of::<Root>()),
        )?;
        self.runtime
            .recover_prepared_output_source(receipt, std::any::TypeId::of::<Root>())
            .ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "the required root publication has no exact retained observation",
                )
            })
    }

    pub fn ensure_recovered_program_root_source_bound<Root>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        source: &WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
            SourceValue<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
        >,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    {
        if !self.contains_output_root::<Root>() {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignDemand,
                "selected output root is not installed for this program",
            ));
        }
        self.runtime
            .ensure_recovered_output_source_bound(prepared, source)
    }

    pub fn validate_recovered_program_root_currentness<Root>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        retained: &WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
            SourceValue<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
        >,
        current: &WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
            SourceValue<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
        >,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    {
        if !self.contains_output_root::<Root>() {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignDemand,
                "selected output root is not installed for this program",
            ));
        }
        self.runtime
            .validate_recovered_output_source_currentness(prepared, retained, current)
    }

    pub fn bind_prepared_program_root_source<Root>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        source: &WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
            SourceValue<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
        >,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    {
        if !self.contains_output_root::<Root>() {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignDemand,
                "selected output root is not installed for this program",
            ));
        }
        self.runtime.bind_prepared_output_source(prepared, source)
    }

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
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    {
        if !self.contains_output_root::<Root>() {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignDemand,
                "selected output root is not installed for this program",
            ));
        }
        self.validate_root_artifact_demand::<Root>(maximum_work, maximum_retained_bytes)?;
        self.runtime.validate_recovered_output_root_kind(
            source_receipt,
            crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind::Required(std::any::TypeId::of::<Root>()),
        )?;
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
                target_feature: std::any::TypeId::of::<<RootConnectionRef<Schema, Root> as ApplicationConnectionShape<Schema>>::TargetFeature>(),
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
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    {
        if !self.contains_output_root::<Root>() {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignDemand,
                "selected output root is not installed for this program",
            ));
        }
        self.validate_root_artifact_demand::<Root>(maximum_work, maximum_retained_bytes)?;
        self.runtime
            .admit_performed_output_demand::<
                Family<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
            >(source, maximum_work, maximum_retained_bytes, prepared)
            .map(|admitted| WorthQueryAdmittedProgramOutput {
                admitted,
                target_feature: std::any::TypeId::of::<<RootConnectionRef<Schema, Root> as ApplicationConnectionShape<Schema>>::TargetFeature>(),
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
        demand: &mut WorthQueryAdmittedProgramOutput<Schema, Program, Demand>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, Demand>,
            SourceValue<Schema, Demand>,
        >,
    ) -> Result<WorthQueryProgramOutputAdvance<Schema, Program, Demand>, WorthQueryOutputDemandDenial>
    where
        Demand: WorthQueryApplicationOutputDemand<Schema>,
        SourceValue<Schema, Demand>: 'static,
        SourceQuery<Schema, Demand>: 'static,
    {
        self.runtime
            .advance_program_output_demand(
                &mut demand.admitted,
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
                        target_feature: demand.target_feature,
                        marker: std::marker::PhantomData,
                    })
                }
            })
    }
}
