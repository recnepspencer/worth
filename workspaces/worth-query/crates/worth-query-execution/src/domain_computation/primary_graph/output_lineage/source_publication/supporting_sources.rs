//! Completion aliases from the actual retained consumed-output closure.
use super::*;
use crate::domain_computation::primary_graph::invariant_projection::EvidenceView;
use im::OrdSet;
use std::sync::Arc;

impl WorthQueryApplicationOutputLineage {
    pub(super) fn supporting_publication_sources(
        &self,
        candidate: &WorthQueryCurrentOutputCandidate,
        aliases: &mut Vec<(SemanticSource, [u8; 32])>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let mut pending = Vec::new();
        let mut visited = OrdSet::new();
        self.push_supports(&candidate.consumed_outputs, &mut pending, admission)?;
        while let Some(evidence) = pending.pop() {
            charge(admission, 1)?;
            crate::domain_computation::primary_graph::output_reuse::owner_read(
                admission.admit_visited_settlement(visited.len()),
                "availability dependency visit",
            )?;
            if visited.contains(evidence.identity()) {
                continue;
            }
            visited.insert(Arc::clone(evidence.identity()));
            // This is the native recorded address, not a caller key. Missing
            // original custody cannot be promoted to a live source partition.
            let identity = evidence.identity();
            let (occurrence, generation, slot) = identity.address();
            let source = identity.source();
            charge(admission, 3)?;
            let record = self
                .by_source
                .get(source)
                .and_then(|rows| rows.get(&occurrence))
                .and_then(|rows| rows.get(&generation))
                .and_then(|rows| rows.get(slot))
                .and_then(|row| row.get())
                .filter(|row| row.settlement_identity.as_ref() == identity.as_ref())
                .ok_or_else(|| stop(Kind::IncompleteDependencyCoverage))?;
            let partition = record
                .source_partition_identity
                .ok_or_else(|| stop(Kind::IncompleteDependencyCoverage))?;
            let mut family = None;
            for bindings in self.output_families.values() {
                for (binding, _) in bindings {
                    charge(admission, 1)?;
                    if self.binding_identity(*binding).as_ref() == Some(&source.output_binding)
                        && family.replace(bindings).is_some()
                    {
                        return Err(stop(Kind::IncompleteDependencyCoverage));
                    }
                }
            }
            let family = family.ok_or_else(|| stop(Kind::ProducerUnavailable))?;
            for (binding, _) in family {
                charge(admission, 1)?;
                crate::domain_computation::primary_graph::invariant_projection::consumed_output::reserve_pending(aliases, 1, admission)
                    .map_err(|stop| crate::domain_computation::primary_graph::output_reuse::verification_denial(stop, "availability completion aliases"))?;
                let mut alias = source.clone();
                alias.output_binding = self
                    .binding_identity(*binding)
                    .ok_or_else(|| stop(Kind::ProducerUnavailable))?;
                aliases.push((alias, partition));
            }
            self.push_supports(evidence.upstream(), &mut pending, admission)?;
        }
        Ok(())
    }
    fn push_supports<'a>(
        &self,
        values: &'a [crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence],
        pending: &mut Vec<EvidenceView<'a>>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        charge(admission, values.len() as u64)?;
        crate::domain_computation::primary_graph::invariant_projection::consumed_output::reserve_pending(pending, values.len(), admission)
            .map_err(|stop| crate::domain_computation::primary_graph::output_reuse::verification_denial(stop, "availability support closure"))?;
        pending.extend(values.iter().map(EvidenceView::from));
        Ok(())
    }
}
