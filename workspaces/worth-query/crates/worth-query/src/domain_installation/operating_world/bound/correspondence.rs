use super::WorthQueryBoundDomainOperation;
use crate::basis_lifecycle::BasisOperationLane;

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane>
    WorthQueryBoundDomainOperation<D, O, F, L>
{
    pub fn semantic_correspondence_registration<G: 'static>(
        &self,
        location: worth_query_installation::facade::WorthQueryConditionalNodeLocation,
        dependency_ordinal: usize,
        graph: &crate::domain_installation::WorthQueryInstalledGraphParticipation<G>,
        source_record_identity: Option<
            worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
        >,
        targets: Vec<worth_runtime_bridge::facade::BridgeSignalAspectTargetDeclaration>,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeSemanticCorrespondenceRegistration,
        worth_runtime_bridge::facade::BridgeCorrespondenceDenial,
    > {
        self.operation.semantic_correspondence_registration(
            location,
            dependency_ordinal,
            graph,
            source_record_identity,
            targets,
        )
    }

    pub fn install_semantic_correspondence<G: 'static>(
        &self,
        location: worth_query_installation::facade::WorthQueryConditionalNodeLocation,
        dependency_ordinal: usize,
        graph_participation: &crate::domain_installation::WorthQueryInstalledGraphParticipation<G>,
        source_record_identity: Option<
            worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
        >,
        graph: &mut worth_runtime_bridge::facade::BridgeSignalGraphBinding<'_, '_>,
    ) -> crate::domain_installation::WorthQueryInstalledSemanticCorrespondenceOutcome<D, O, F, G>
    {
        self.operation.install_semantic_correspondence(
            location,
            dependency_ordinal,
            graph_participation,
            source_record_identity,
            graph,
        )
    }
}
