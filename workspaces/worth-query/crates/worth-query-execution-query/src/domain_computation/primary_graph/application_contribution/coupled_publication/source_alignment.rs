//! Aligned managed Source reads (#310): a decision of this assembly read a
//! managed Parent from its committed basis because Native proved the
//! candidate left it unchanged. Every later append is judged against each
//! such read by Native, from that member's own records, and a member that
//! changed one marks it at the image it produced. The mark is the read's
//! candidate support for reopen qualification and final coupled support.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    AppliedSourceAlignment, WorthQueryApplicationAttemptDenial as Attempt,
};
use worth_query_execution_lineage::facade::ports::PrimaryGraphInternalPort as _;
use worth_relational::facade::runtime::{
    DerivedRelationReadDenial as Read, ManagedIndexSourcePreparationDenial as Native,
};

impl<S: ApplicationSchema> CoupledApplicationAssembly<'_, '_, '_, S> {
    pub(super) fn retain_alignment(&self, alignment: std::sync::Arc<AppliedSourceAlignment>) {
        self.alignments.borrow_mut().push(alignment);
    }

    /// Mark each retained aligned read the member that just appended changed.
    /// A read Native cannot judge is marked: equality is never manufactured.
    pub(super) fn mark_alignments(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        member: &qualification::AppendedMember,
    ) -> Result<(), CoupledSourceReadStop> {
        let alignments = self.alignments.borrow();
        if alignments.is_empty() {
            return Ok(());
        }
        let touches = match member {
            qualification::AppendedMember::Part { index, .. } => {
                candidate.part_append_touches(execution, *index)?
            }
            qualification::AppendedMember::Stage { physical, .. } => {
                candidate.authored_stage_append_touches(execution, *physical)?
            }
        };
        let current = candidate.pin_source(execution)?;
        let graph = self.application.runtime.primary_graph();
        for alignment in alignments.iter() {
            let unchanged = match graph
                .and_then(|graph| graph.layout_ref().managed_source(alignment.source()))
            {
                None => false,
                Some(program) => match candidate.managed_source_unchanged_by(
                    execution,
                    program,
                    &current,
                    alignment.scope(),
                    alignment.footprint(),
                    touches.records(),
                ) {
                    Ok(unchanged) => unchanged,
                    Err(Native::Source(Read::Request(cause))) => {
                        return Err(CoupledSourceReadStop::capture(
                            Attempt::relational_read_stopped(cause, "aligned managed Source"),
                        ))
                    }
                    Err(_) => false,
                },
            };
            if !unchanged {
                alignment.mark_changed(current.candidate_identity());
            }
        }
        Ok(())
    }
}
