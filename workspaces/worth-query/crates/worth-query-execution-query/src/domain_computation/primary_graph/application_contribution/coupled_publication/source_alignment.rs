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
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};

type Image = RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>;

/// The retained aligned reads and the candidate image the last judged
/// member appended to: the image a read was proved unchanged at.
#[derive(Default)]
pub(super) struct Alignments {
    reads: Vec<std::sync::Arc<AppliedSourceAlignment>>,
    image: Option<Image>,
}

impl Alignments {
    pub(super) fn retain(&mut self, read: std::sync::Arc<AppliedSourceAlignment>, observed: &Image) {
        self.reads.push(read);
        // A read is made at the assembly's current image, the latest known.
        let later = self.image.as_ref().is_none_or(|image| {
            observed
                .candidate_identity()
                .includes_assembly_prefix(image.candidate_identity())
        });
        if later {
            self.image = Some(observed.clone());
        }
    }
}


impl<S: ApplicationSchema> CoupledApplicationAssembly<'_, '_, '_, S> {
    /// Mark each retained aligned read the member that just appended changed.
    /// A read Native cannot judge is marked: equality is never manufactured.
    pub(super) fn mark_alignments(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        member: &qualification::AppendedMember,
    ) -> Result<(), CoupledSourceReadStop> {
        let mut alignments = self.alignments.borrow_mut();
        if alignments.reads.is_empty() {
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
        for alignment in &alignments.reads {
            let program =
                graph.and_then(|graph| graph.layout_ref().managed_source(alignment.source()));
            let unchanged = match (program, &alignments.image) {
                (Some(program), Some(earlier)) => match candidate.managed_source_unchanged_by(
                    execution,
                    program,
                    &current,
                    earlier,
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
                _ => false,
            };
            if !unchanged {
                alignment.mark_changed(current.candidate_identity());
            }
        }
        alignments.image = Some(current);
        Ok(())
    }
}
