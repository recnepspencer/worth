use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition,
};
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationStructuredValueBinding,
};

use super::{WorthQueryAdmittedProgramOutput, WorthQueryProgramApplicationRuntime};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationDiscoveredOutputConnection, WorthQueryApplicationOutputDemand,
    WorthQueryApplicationOutputDemandSource, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind, WorthQueryPreparedRequiredOutputSource,
    WorthQueryProducerOutputFamily,
};

type RootConnection<Schema, Root> =
    <<Root as ApplicationOutputGraphShape<Schema>>::RootConnection as ApplicationConnectionShape<
        Schema,
    >>::Binding;
pub type WorthQueryDiscoveredProgramRootDemand<Schema, Root> =
    <RootConnection<Schema, Root> as WorthQueryApplicationDiscoveredOutputConnection<Schema>>::Demand;
type Family<Schema, Root> =
    <WorthQueryDiscoveredProgramRootDemand<Schema, Root> as WorthQueryApplicationOutputDemand<
        Schema,
    >>::OutputFamily;
type Source<Schema, Root> =
    <Family<Schema, Root> as WorthQueryProducerOutputFamily<Schema>>::Source;
type SourceQuery<Schema, Root> = <Source<Schema, Root> as ApplicationQueryBinding<Schema>>::Query;
type SourceValue<Schema, Root> =
    <<Source<Schema, Root> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
{
    /// Checks retained discovered-source custody before a detached continuation
    /// reads or mutates any program state. This validates existing authority;
    /// it neither duplicates the prepared token nor recovers roots from a receipt.
    pub fn validate_discovered_program_source<Root>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        observation: &crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
    {
        self.require_discovered_root::<Root>()?;
        if prepared.runtime_authority != self.runtime.runtime.authority_identity().as_u64()
            || &prepared.source_commit != receipt.committed_product_publication().composite_commit()
            || &prepared.source_commit != observation.selected_commit()
            || prepared.product_occurrence != observation.branch_incarnation()
            || branch.occurrence() != observation.branch_incarnation()
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "discovered continuation differs from its original runtime or publication",
            ));
        }
        self.runtime.output_demands.validate_recovery_root_kind(
            receipt,
            crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind::Discovered(std::any::TypeId::of::<Root>()),
        )?;
        self.runtime
            .select_application_read_observation(observation)
            .map_err(|denial| {
                WorthQueryOutputDemandDenial::product_selection(
                    denial,
                    "discovered continuation observation",
                )
            })?;
        use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
        let kind = match request.interruption() {
            Some(WorthQueryRequestInterruption::Cancelled) => Some(WorthQueryOutputDemandDenialKind::Cancelled),
            Some(WorthQueryRequestInterruption::DeadlineExceeded) => Some(WorthQueryOutputDemandDenialKind::TimedOut),
            None if self.runtime.authentication_is_expired(principal.valid_until()) => Some(
                WorthQueryOutputDemandDenialKind::SourcePrincipal(
                    crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenialKind::ExpiredAuthentication)),
            None => None,
        };
        match kind {
            Some(kind) => Err(WorthQueryOutputDemandDenial::new(
                kind,
                "discovered continuation request",
            )),
            None => Ok(()),
        }
    }

    pub fn retain_discovered_program_source<Root>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        prepared: &WorthQueryPreparedRequiredOutputSource,
    ) -> Result<WorthQueryPreparedRequiredOutputSource, WorthQueryOutputDemandDenial>
    where
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
    {
        self.require_discovered_root::<Root>()?;
        if prepared.runtime_authority != self.runtime.runtime.authority_identity().as_u64() {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "the program source belongs to another runtime",
            ));
        }
        self.runtime.output_demands.retain_program_source(
            prepared,
            crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind::Discovered(std::any::TypeId::of::<Root>()),
        )
    }

    pub fn validate_recovered_discovered_program_root_currentness<Root>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        retained: &WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, Root>,
            SourceValue<Schema, Root>,
        >,
        current: &WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, Root>,
            SourceValue<Schema, Root>,
        >,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
    {
        self.require_discovered_root::<Root>()?;
        self.runtime
            .validate_recovered_output_source_currentness(prepared, retained, current)
    }

    pub fn bind_prepared_discovered_program_root_sources<Root>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        sources: &[WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, Root>,
            SourceValue<Schema, Root>,
        >],
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
    {
        self.require_discovered_root::<Root>()?;
        self.runtime.bind_prepared_output_sources(prepared, sources)
    }

    pub fn admit_performed_discovered_program_root_output<Root>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        source: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, Root>,
            SourceValue<Schema, Root>,
        >,
        limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
        prepared: &WorthQueryPreparedRequiredOutputSource,
    ) -> Result<
        WorthQueryAdmittedProgramOutput<
            Schema,
            Program,
            WorthQueryDiscoveredProgramRootDemand<Schema, Root>,
        >,
        WorthQueryOutputDemandDenial,
    >
    where
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
    {
        self.require_discovered_root::<Root>()?;
        let artifact = self.validate_derived_artifact_demand::<
            <Root as ApplicationOutputGraphShape<Schema>>::RootConnection,
            WorthQueryDiscoveredProgramRootDemand<Schema, Root>,
        >()?;
        let limits = self.resolve_artifact_limits(limits, artifact)?;
        self.runtime
            .admit_performed_output_demand::<Family<Schema, Root>>(
                source,
                limits,
                prepared,
            )
            .map(|admitted| WorthQueryAdmittedProgramOutput {
                admitted,
                target_feature: std::any::TypeId::of::<<<Root as ApplicationOutputGraphShape<Schema>>::RootConnection as ApplicationConnectionShape<Schema>>::TargetFeature>(),
                artifact,
                marker: std::marker::PhantomData,
            })
    }

    pub fn recover_discovered_program_root_output<Root>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        source: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, Root>,
            SourceValue<Schema, Root>,
        >,
        current: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, Root>,
            SourceValue<Schema, Root>,
        >,
        limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
        source_receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Result<
        WorthQueryAdmittedProgramOutput<
            Schema,
            Program,
            WorthQueryDiscoveredProgramRootDemand<Schema, Root>,
        >,
        WorthQueryOutputDemandDenial,
    >
    where
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
    {
        self.require_discovered_root::<Root>()?;
        let artifact = self.validate_derived_artifact_demand::<
            <Root as ApplicationOutputGraphShape<Schema>>::RootConnection,
            WorthQueryDiscoveredProgramRootDemand<Schema, Root>,
        >()?;
        let limits = self.resolve_artifact_limits(limits, artifact)?;
        self.runtime.validate_recovered_output_root_kind(
            source_receipt,
            crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind::Discovered(std::any::TypeId::of::<Root>()),
        )?;
        self.runtime
            .admit_recovered_output_demand::<Family<Schema, Root>>(
                source,
                current,
                limits,
                source_receipt,
            )
            .map(|admitted| WorthQueryAdmittedProgramOutput {
                admitted,
                target_feature: std::any::TypeId::of::<<<Root as ApplicationOutputGraphShape<Schema>>::RootConnection as ApplicationConnectionShape<Schema>>::TargetFeature>(),
                artifact,
                marker: std::marker::PhantomData,
            })
    }

    fn require_discovered_root<Root>(&self) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
    {
        if self.contains_output_root::<Root>() {
            Ok(())
        } else {
            Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                "selected discovered output root is not installed for this program",
            ))
        }
    }
}
