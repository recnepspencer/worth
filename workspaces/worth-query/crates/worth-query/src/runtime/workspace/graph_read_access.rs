use super::WorthQueryWorkspace;

impl WorthQueryWorkspace {
    pub fn graph_index_inventory(&self) -> super::super::WorthQueryGraphIndexInventory {
        self.runtime.graph_index_inventory()
    }

    pub(crate) fn admit_graph_read_access_for_family(
        &self,
        family: &super::super::WorthQueryReadFamily,
    ) -> Result<
        super::super::WorthQueryGraphReadAccessAdmission,
        super::super::WorthQueryGraphReadAccessShapeExplanationError,
    > {
        self.runtime.admit_graph_read_access_for_family(family)
    }

    pub(crate) fn admit_graph_read_access_for_family_in_authority(
        &self,
        family: &super::super::WorthQueryReadFamily,
        authority: &super::super::WorthQueryGraphReadAccessAuthorityContext,
    ) -> Result<
        super::super::WorthQueryGraphReadAccessAdmission,
        super::super::WorthQueryGraphReadAccessShapeExplanationError,
    > {
        self.runtime
            .admit_graph_read_access_for_family_in_authority(family, authority)
    }
}
