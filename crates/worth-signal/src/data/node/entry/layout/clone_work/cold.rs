//! Selected cold-node diagnostic payload copy admission.
use super::{comparator, scopes_copy, sequence, string};
use crate::data::error::SignalError;
use crate::data::node::NodeColdData;
use crate::logic::evaluation::EvaluationWork;

impl NodeColdData {
    pub(crate) fn admit_clone_work(
        &self,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        work.reserve(Some(std::mem::size_of::<Self>() + 32))?;
        let Self {
            retained_artifact,
            causality,
            execution_trace: _,
        } = self;
        if let Some(artifact) = retained_artifact {
            let crate::data::trace::RetainedDiagnosticArtifact {
                changed_regions,
                labels,
                keyed_family,
                keyed_key,
                reuse_certification,
                reuse_boundary_context,
            } = artifact;
            sequence::<crate::data::output::ChangedRegion>(changed_regions.as_slice().len(), work)?;
            for region in changed_regions.as_slice() {
                string(Some(&region.partition.0), work)?;
                string(region.detail.as_deref(), work)?;
            }
            sequence::<String>(labels.len(), work)?;
            for label in labels {
                string(Some(label), work)?;
            }
            string(keyed_family.as_deref(), work)?;
            string(keyed_key.as_deref(), work)?;
            if let Some(certification) = reuse_certification {
                sequence::<crate::data::reuse::ReuseBoundaryProof>(
                    certification.proofs.len(),
                    work,
                )?;
            }
            if let Some(context) = reuse_boundary_context {
                context_copy(context, work)?;
            }
        }
        if let Some(causality) = causality {
            string(Some(&causality.kind), work)?;
            sequence::<(String, String)>(causality.fields.len(), work)?;
            for (key, value) in &causality.fields {
                string(Some(key), work)?;
                string(Some(value), work)?;
            }
        }
        Ok(())
    }
}

fn context_copy(
    context: &crate::data::reuse::ReuseBoundaryContext,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    use crate::data::reuse::{PersistentCorrespondenceEvidence, ReuseStrategyBoundaryContext};
    let crate::data::reuse::ReuseBoundaryContext {
        topology_regime: _,
        tolerance_regime,
        semantic_region,
        authority_policy: _,
        artifact_family,
        structural_dependency_basis: _,
        partition_region_basis,
        strategy_detail,
    } = context;
    comparator(tolerance_regime, work)?;
    string(artifact_family.as_ref().map(|v| v.as_str()), work)?;
    scopes_copy(&semantic_region.partition_scope, work)?;
    scopes_copy(partition_region_basis.as_slice(), work)?;
    match strategy_detail {
        ReuseStrategyBoundaryContext::None => Ok(()),
        ReuseStrategyBoundaryContext::PartialArtifactSplice {
            composition_regions,
        } => scopes_copy(composition_regions.as_slice(), work),
        ReuseStrategyBoundaryContext::CrossIdentity {
            persistent_correspondence,
        } => {
            let (PersistentCorrespondenceEvidence::HostSuppliedKey(value)
            | PersistentCorrespondenceEvidence::ContractDeclaredBasis(value)
            | PersistentCorrespondenceEvidence::LineageBackedMapping(value)
            | PersistentCorrespondenceEvidence::RegionIdentityBasis(value)) =
                persistent_correspondence;
            string(Some(value), work)
        }
    }
}
