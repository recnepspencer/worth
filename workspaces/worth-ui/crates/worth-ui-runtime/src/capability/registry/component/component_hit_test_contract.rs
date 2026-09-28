use super::{
    ComponentAllocationMeasurementContract, ComponentHitTestClipContract, ComponentHitTestInset,
    ComponentHitTestOrder,
};

/// Complete component-owned meaning for one allocation-bounded hit region.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentHitTestContract {
    order: ComponentHitTestOrder,
    allocation: ComponentAllocationMeasurementContract,
    clip: ComponentHitTestClipContract,
}

impl ComponentHitTestContract {
    pub const fn allocation_bounds(
        order: ComponentHitTestOrder,
        allocation: ComponentAllocationMeasurementContract,
    ) -> Self {
        Self {
            order,
            allocation,
            clip: ComponentHitTestClipContract::allocation_bounds(),
        }
    }

    pub const fn allocation_bounds_clipped_by_inset(
        order: ComponentHitTestOrder,
        allocation: ComponentAllocationMeasurementContract,
        inset: ComponentHitTestInset,
    ) -> Self {
        Self {
            order,
            allocation,
            clip: ComponentHitTestClipContract::inset(inset),
        }
    }

    pub const fn order(self) -> ComponentHitTestOrder {
        self.order
    }

    pub const fn allocation(self) -> ComponentAllocationMeasurementContract {
        self.allocation
    }

    pub const fn clip(self) -> ComponentHitTestClipContract {
        self.clip
    }

    pub(crate) fn digest_basis(self) -> String {
        let legacy = format!(
            "allocation-bounds:{}:{}",
            self.order.rank(),
            self.allocation.digest_basis()
        );
        match self.clip {
            ComponentHitTestClipContract::AllocationBounds => legacy,
            ComponentHitTestClipContract::Inset(_) => {
                format!("{legacy}:{}", self.clip.digest_basis())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ComponentHitTestContract;
    use crate::capability::{
        ComponentAllocationMeasurementContract, ComponentHitTestInset, ComponentHitTestOrder,
    };

    #[test]
    fn clipped_contract_changes_identity_without_churning_legacy_bounds_identity() {
        let allocation = ComponentAllocationMeasurementContract::fill_viewport();
        let legacy = ComponentHitTestContract::allocation_bounds(
            ComponentHitTestOrder::front_to_back(3),
            allocation,
        );
        let clipped = ComponentHitTestContract::allocation_bounds_clipped_by_inset(
            ComponentHitTestOrder::front_to_back(3),
            allocation,
            ComponentHitTestInset::symmetric(4, 2),
        );

        assert_eq!(legacy.digest_basis(), "allocation-bounds:3:fill-viewport");
        assert_ne!(legacy.digest_basis(), clipped.digest_basis());
    }
}
