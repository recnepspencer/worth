//! Recover only an existing registered fixed-root source.
use super::*;
impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Outputs:
        worth_query_declaration::facade::application_program::ApplicationProgramOutputsShape<
            Schema,
        >,
{
    pub fn recover_prepared_program_root_source<Root>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        selector: crate::domain_computation::primary_graph::WorthQueryOutputSourceReplay<'_>,
    ) -> Result<
        (
            crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
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
        let receipt = self.backing.runtime().resolve_output_source_replay(
            phase,
            selector,
            crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind::Required(TypeId::of::<Root>()),
        )?;
        let (observation, prepared) = self.backing.runtime()
            .recover_prepared_output_source(&receipt, TypeId::of::<Root>())
            .ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "the required root publication has no exact retained observation",
                )
            })?;
        Ok((receipt, observation, prepared))
    }
}
