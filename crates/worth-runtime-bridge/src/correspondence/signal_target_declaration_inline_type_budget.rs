//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<BridgeSignalAspectTargetDeclaration>(
            "crates/worth-runtime-bridge/src/correspondence/signal_target_declaration.rs",
            "BridgeSignalAspectTargetDeclaration",
            128 * 1024,
        );
        owner.field::<BridgeAspectRegistrationId>("aspect_registration_id");
        owner.field::<Arc<InstalledSignalNodeCapability>>("node_capability");
        owner.field::<PartitionToken>("partition");
        owner.field::<NodeId>("node");
        owner.field::<BridgeSignalSlotRequest>("slot");
    }
}
