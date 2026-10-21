//! Native posting projection of the actual retained producer closure.
use super::*;
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
use crate::domain_computation::primary_graph::invariant_projection::EvidenceView;
use crate::domain_computation::primary_graph::output_lineage::invalidation::RegisteredOutputFacts;
use im::OrdSet;

pub(super) struct PostingDependencies {
    sources: Vec<FactPayload>,
    outputs: Vec<RegisteredOutputFacts>,
}
impl PostingDependencies {
    pub(super) fn capture(
        candidate: &CandidateEvidence,
        owner: &SourceInvalidationOwner,
    ) -> Result<Self, Stop> {
        let facts = candidate
            .observed_source_facts
            .for_comparison()
            .ok_or(Stop::Unavailable)?;
        let mut result = Self {
            sources: Vec::new(),
            outputs: Vec::new(),
        };
        result.push(
            facts.facts().clone(),
            candidate
                .native_output_witness
                .as_ref()
                .and_then(|w| w.get()),
            owner,
        )?;
        let mut pending = Vec::new();
        let mut visited = OrdSet::new();
        push_pending(&candidate.consumed_outputs, &mut pending)?;
        while let Some(evidence) = pending.pop() {
            if visited.contains(evidence.identity()) {
                continue;
            }
            visited.insert(Arc::clone(evidence.identity()));
            result.push(
                evidence.source_facts().clone(),
                evidence.native_output_witness(),
                owner,
            )?;
            push_pending(evidence.upstream(), &mut pending)?;
        }
        Ok(result)
    }
    fn push(
        &mut self,
        source: FactPayload,
        witness: Option<
            &crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness,
        >,
        owner: &SourceInvalidationOwner,
    ) -> Result<(), Stop> {
        super::super::consumed_output::reserve_pending(&mut self.sources, 1)?;
        super::super::consumed_output::reserve_pending(&mut self.outputs, 1)?;
        let output = owner
            .prepare_performed_output_facts(witness.ok_or(Stop::Unavailable)?)
            .map_err(super::super::consumed_output::map_admission_stop)?
            .ok_or(Stop::Unavailable)?;
        self.sources.push(source);
        self.outputs.push(output);
        Ok(())
    }

    pub(super) fn facts(
        &self,
    ) -> impl Iterator<Item = &crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>
    {
        self.sources
            .iter()
            .flat_map(|source| source.iter())
            .chain(self.outputs.iter().flat_map(|output| output.facts().iter()))
    }
}
fn push_pending<'a>(
    values: &'a [ConsumedOutputEvidence],
    pending: &mut Vec<EvidenceView<'a>>,
) -> Result<(), Stop> {
    super::super::consumed_output::reserve_pending(pending, values.len())?;
    pending.extend(values.iter().map(EvidenceView::from));
    Ok(())
}
