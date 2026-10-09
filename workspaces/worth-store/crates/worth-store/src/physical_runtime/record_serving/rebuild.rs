use worth_store_layout_indexes::{access_shapes, AccessLaneClassification, AccessShapeContract};

/// Store-private carried declaration for a reconstructive selected-root walk.
/// This is not an ordinary caller's permission to widen a foreground read.
#[derive(Clone, Copy)]
pub(in crate::physical_runtime) struct RebuildReadShape {
    contract: AccessShapeContract,
}

impl RebuildReadShape {
    pub(in crate::physical_runtime::record_serving) fn admit() -> Self {
        Self {
            contract: access_shapes()
                .rebuild_read_declaration(AccessLaneClassification::Maintenance)
                .expect("the layout owner admits rebuild reads only in its maintenance lane"),
        }
    }

    pub(in crate::physical_runtime) const fn contract(self) -> AccessShapeContract {
        self.contract
    }
}
