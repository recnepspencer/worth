use std::sync::Arc;
use worth_foundational::facade::{DurableWorldOwnerIdentity, WorldOwnerSlotIdentity};
use worth_runtime_world::facade::WorldDurableHomeBinding;
use worth_store::physical_runtime::ServingPhysicalRuntime;
pub(super) fn attach_bridge(
    section: &worth_store_relational::RelationalStoreSection,
    builder: worth_runtime_bridge::facade::UnattachedRelationalBridgeBuilder,
    graph_role: String,
) -> Result<
    worth_runtime_world::facade::RuntimeWorldCorrespondencePort,
    crate::ApplicationHomeOpenRefusal,
> {
    let source = worth_runtime_bridge::facade::RuntimeBridgeRelationalSource::for_graph_role(
        section.world_composition_runtime(),
        graph_role,
    )
    .map_err(crate::ApplicationHomeOpenRefusal::BridgeSource)?;
    let bridge = builder
        .with_relational_source(source)
        .build()
        .map_err(crate::ApplicationHomeOpenRefusal::Bridge)?;
    Ok(bridge.runtime_world_correspondence_port())
}
/// Actual admitted Home custody. No Clone, raw identity constructor or second
/// extraction exists outside the Home's private composition.
pub(crate) struct UnclaimedWorldSlot {
    medium: super::ApplicationWorldMedium,
}
impl UnclaimedWorldSlot {
    pub(crate) fn from_admitted_home(
        serving: Arc<ServingPhysicalRuntime>,
        descriptor: &crate::descriptor::HomeDescriptor,
    ) -> Self {
        let owner = DurableWorldOwnerIdentity::from_recorded(
            descriptor.identity,
            WorldOwnerSlotIdentity::from_recorded(0),
        );
        Self {
            medium: super::ApplicationWorldMedium {
                serving,
                binding: WorldDurableHomeBinding::from_admitted_adapter(
                    owner,
                    descriptor.incarnation,
                    *descriptor.binding.selected().revision().bytes(),
                ),
                placement: crate::open::physical_admission::placement(),
            },
        }
    }
    pub(in crate::world_section) fn into_medium(self) -> super::ApplicationWorldMedium {
        self.medium
    }
}
