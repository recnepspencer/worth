use worth_ui_host_contract::UiMountedInstanceIdentity;

use super::UiMountedIdentityState;

pub(in crate::mounting) struct UiPresentedAuthoredAttribution {
    pub(in crate::mounting) source_provenance_digest: u64,
    pub(in crate::mounting) semantic_identity_digest: u64,
}

impl UiMountedIdentityState {
    pub(in crate::mounting) fn current_authored_attribution(
        &self,
        mounted_instance: UiMountedInstanceIdentity,
    ) -> Option<UiPresentedAuthoredAttribution> {
        let (node, _) = self
            .current_projection()?
            .semantic_projection()
            .node_receipt_with_probes(mounted_instance);
        let graph_node = node?.graph_node();
        let trace = self.current_trace_source.as_ref()?;
        let artifact_index = trace
            .graph_node_evidence_index()
            .lookup_graph_node_identity(graph_node)?
            .neighborhood()
            .declaration_artifact_index();
        let artifact = trace.declaration_artifacts().get(artifact_index)?;
        let source = artifact.provenance().source_provenance();
        Some(UiPresentedAuthoredAttribution {
            source_provenance_digest: crate::declaration::authored_source_provenance_digest(
                source.module_path(),
                source.declaration_index(),
            ),
            semantic_identity_digest: crate::declaration::stable_text_digest(
                artifact.identity().authored_semantic_name(),
            ),
        })
    }
}
