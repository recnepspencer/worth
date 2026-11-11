//! A row a graph selection transported is current only through that commit's read evidence.
use super::*;
use crate::domain_computation::primary_graph::output_lineage::current_output::retained_candidates::WorthQueryRetainedOutputCandidate;

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Exact reuse of a transported row. Its new settlement has no mark row,
    /// so the read evidence is its whole fence: the committed selection must be
    /// the actual graph selection over the recorded selected basis, and the
    /// producing build must be current. Each recorded fact and witness aspect
    /// must hold, or be a restored member whose original revision the evidence
    /// proves at the selected step and unchanged since the restoring commit.
    /// A row that was stale at the selected step therefore fails here. The
    /// ordinary fence never sees this evidence; any failure selects Preserve.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn restored_output_is_current<
        Family,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
        candidate: &WorthQueryRetainedOutputCandidate,
    ) -> Result<bool, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let (_, installed) = self
            .installed_producers
            .select_exact::<Family>(candidate.binding)?;
        let build = worth_relational::facade::mvcc::RequiredOutputProducingImplementation::new(
            installed.edition.build_identity(),
            installed.edition.digest(),
        );
        if candidate.producing_implementation != Some(build) {
            return Ok(false);
        }
        let Some(read) = crate::domain_computation::primary_graph::output_reuse::HistoricalOutputRead::from_candidate(candidate) else { return Ok(false); };
        read.is_current(execution, runtime, selected)
    }
}
