use crate::physical_runtime::record_serving::arena::ReleasedControlArenaPlacement;

use super::{PhysicalMutationAdmissionDisposition, PreparedPhysicalMutation};

impl PreparedPhysicalMutation {
    pub(in crate::physical_runtime::record_serving) fn with_released_control_placement(
        mut self,
        placement: ReleasedControlArenaPlacement,
    ) -> Self {
        assert_eq!(
            self.disposition(),
            PhysicalMutationAdmissionDisposition::Fresh
        );
        assert!(self.released_control_placement.is_none());
        self.released_control_placement = Some(placement);
        self
    }

    pub(in crate::physical_runtime::record_serving) fn released_control_placement(
        &self,
    ) -> Option<&ReleasedControlArenaPlacement> {
        self.released_control_placement.as_ref()
    }

    pub(in crate::physical_runtime::record_serving) fn take_released_control_placement(
        &mut self,
    ) -> Option<ReleasedControlArenaPlacement> {
        self.released_control_placement.take()
    }
}
