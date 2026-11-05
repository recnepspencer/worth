use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::*;
    pub(super) fn decide<Schema: TopologyModelBinding>(
        _: &mut crate::domain_computation::primary_graph::output_lineage::ownership_oracle::region_output::Decision<'_, '_, '_, Schema>,
    ) -> Result<
        Option<(
            WorthQueryInvariantMutationTarget<Schema, crate::domain_computation::primary_graph::output_lineage::ownership_oracle::facts::SetEntry>,
            u64,
        )>,
        HandlerExecutionDenial,
>{
    Ok(None)
}
