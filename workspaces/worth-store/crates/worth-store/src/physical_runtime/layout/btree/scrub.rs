use super::lookup::PhysicalBTreeIndex;
use crate::physical_runtime::{
    integrity::SelectedRecordScrubBasis, PhysicalIntegrityScrubRequestDenial,
    PhysicalIntegrityScrubTarget,
};

impl PhysicalBTreeIndex<'_, '_> {
    /// Issues a diagnostic target only after this index encountered C.9 damage
    /// or registered-family shape damage while walking its Store-selected D.1
    /// root or a validated child edge.
    /// The retained scope is descriptive, not authority to bypass C.5 routing.
    pub fn damaged_node_scrub_target(
        &self,
    ) -> Result<Option<PhysicalIntegrityScrubTarget>, PhysicalIntegrityScrubRequestDenial> {
        let Some(scope) = self.damaged_node.get() else {
            return Ok(None);
        };
        let (key_bytes, leaf_value_bytes) = self.registered.cell_shape();
        PhysicalIntegrityScrubTarget::selected_record(
            scope,
            self.port.reader().protected_root(),
            SelectedRecordScrubBasis::BTreeNode {
                family_code: self.registered.family_code(),
                key_bytes,
                leaf_value_bytes,
            },
        )
        .map(Some)
    }
}
