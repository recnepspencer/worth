//! Prepare a recovered original's successor again from its recorded planned leg.
use super::*;
use crate::conditional_execution::BridgePlannedDefinitionLeg;

impl BridgeReadmittedAuthoredAssembly {
    /// The same successor, prepared at the leg's recorded node by this
    /// program's installed decoders. A leg this program does not prepare
    /// byte-for-byte refuses before any Signal effect.
    pub fn prepare_planned_definition_publication(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        leg: &BridgePlannedDefinitionLeg,
    ) -> Result<BridgePreparedAuthoredDefinitionPublication, D> {
        let parts = leg
            .decode(execution)
            .map_err(|cause| D::Section(Box::new(cause)))?;
        let node = parts.signal.node();
        let branch = node.branch();
        let predecessor = self.installed_lowering_at(branch, node)?;
        let providers = predecessor
            .providers
            .reconstruct_authored(&parts.providers.descriptions, execution)
            .map_err(|cause| D::Section(Box::new(BridgeAuthoredSectionDenial::Provider(cause))))?;
        let request = BridgeOwnedConditionalInstallationRequest {
            contract: parts.contract,
            location: parts.location,
            dependencies: predecessor.semantic_dependencies().cloned().collect(),
            providers,
        };
        let prepared = self.prepare_conditional_definition_publication(
            execution,
            branch,
            &predecessor,
            request,
        )?;
        if prepared.planned_leg() != leg {
            return Err(D::PlannedLegChanged);
        }
        Ok(prepared)
    }

    /// The serving lowering installed at exactly this recorded node.
    fn installed_lowering_at(
        &self,
        branch: DurableSignalBranchIdentity,
        node: worth_foundational::facade::DurableSignalNodeIdentity,
    ) -> Result<Arc<BridgeInstalledConditionalLowering>, D> {
        let projection = self.branches.projection(branch).map_err(D::Branch)?;
        let serving = projection.serving(branch).map_err(D::Branch)?;
        let lowerings = serving
            .lowerings
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        lowerings
            .values()
            .find(|lowering| {
                lowering.signal_node().index() == node.index()
                    && lowering.signal_node().generation() == node.generation()
            })
            .cloned()
            .ok_or(D::NotCurrent {
                branch,
                node: worth_signal::facade::NodeId::new(node.index(), node.generation()),
            })
    }
}
