//! Native negative completion of a fixed leaf root, never a partial output tree.
use super::*;
use worth_query_declaration::facade::application_program::{
    ApplicationOutputLeaf, ApplicationRequiredOutputRoot,
};
impl<Schema: ApplicationSchema + 'static, Program: ApplicationProgramDefinition<Schema>>
    WorthQueryProgramApplicationRuntime<Schema, Program>
{
    /// Certifies the original failed root and releases only its source lifecycle.
    /// No successful output or publication is created by this operation.
    pub fn finish_unavailable_program_output<Root>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        demand: &mut WorthQueryAdmittedProgramOutput<
            Schema,
            Program,
            WorthQueryProgramRootDemand<Schema, Root>,
        >,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
        disclosure: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
            SourceValue<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
        >,
    ) -> Result<WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenial>
    where
        Root: ApplicationOutputGraphShape<Schema, Dependents = ApplicationOutputLeaf>
            + ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    {
        if !self.contains_output_root::<Root>() || demand.target_feature != std::any::TypeId::of::<<RootConnectionRef<Schema, Root> as ApplicationConnectionShape<Schema>>::TargetFeature>() {
            return Err(WorthQueryOutputDemandDenial::new(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignDemand, "unavailable completion differs from the installed root"));
        }
        self.runtime.output_demands.validate_owned_root_kind(prepared,
            crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind::Required(std::any::TypeId::of::<Root>()))?;
        self.runtime.finish_unavailable_output_demand(
            phase,
            &mut demand.admitted,
            prepared,
            principal,
            scope,
            branch,
            disclosure,
        )
    }
}
