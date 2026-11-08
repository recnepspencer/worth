//! Independent Native comparison when no derived mark image covers the read.
use super::*;

impl FullVerificationImage<'_> {
    pub(super) fn verify_native_evidence(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        root: EvidenceView<'_>,
    ) -> Result<FullVerificationDecision, FullVerificationStop> {
        let mut pending = Vec::new();
        reserve_pending(&mut pending, 1)?;
        pending.push(root);
        self.verify_native_sources(execution, pending, None)
    }
    pub(super) fn verify_native_sources(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mut pending: Vec<EvidenceView<'_>>,
        historical: Option<&worth_relational::facade::runtime::RelationalHistoricalSelectionRead>,
    ) -> Result<FullVerificationDecision, FullVerificationStop> {
        let mut visited = OrdSet::new();
        let current = if historical.is_some() {
            Some(
                self.runtime
                    .read_truth()
                    .project_snapshot(self.snapshot)
                    .ok_or(FullVerificationStop::SourceUnavailable)?,
            )
        } else {
            None
        };
        while let Some(source) = pending.pop() {
            if visited.contains(source.identity()) {
                continue;
            }
            visited.insert(Arc::clone(source.identity()));
            let source_current = match historical {
                Some(evidence) => self.historical_source_is_current(
                    execution,
                    source,
                    evidence,
                    current
                        .as_ref()
                        .expect("historical verification projects its exact current snapshot"),
                )?,
                None => self.root_source_is_current(execution, source)?,
            };
            if !source_current {
                return Ok(FullVerificationDecision::Changed);
            }
            let witness = source
                .native_output_witness()
                .ok_or(FullVerificationStop::OutputEvidenceUnavailable)?;
            let output_current = match historical {
                Some(evidence) => witness
                    .unchanged_since_restoration(
                        self.runtime,
                        self.snapshot,
                        execution,
                        current
                            .as_ref()
                            .expect("historical verification projects its exact current snapshot"),
                        evidence,
                    )?
                    .map_err(super::historical_selection::evidence_stop)?,
                None => witness.unchanged_in(self.runtime, self.snapshot, execution)?,
            };
            if !output_current {
                return Ok(FullVerificationDecision::Changed);
            }
            reserve_pending(&mut pending, source.upstream().len())?;
            pending.extend(source.upstream().iter().map(EvidenceView::from));
        }
        Ok(FullVerificationDecision::Current)
    }
}
