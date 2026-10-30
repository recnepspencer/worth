use worth_foundational::facade::{
    AspectKey, AspectLocator, AspectMask, LocatorAuthority, MutationMask, ProjectionMask,
};
use worth_runtime_bridge::facade::{BridgeCommittedPatchTarget, TruthDeltaSurfaceKind};

fn main() {
    let _target = BridgeCommittedPatchTarget {
        aspect_locator: AspectLocator::new(
            LocatorAuthority::Authoritative,
            AspectKey::new("profile").expect("valid native aspect key"),
        ),
        field_locator: None,
        mutation_mask: AspectMask::<MutationMask>::whole_aspect(),
        projection_mask: AspectMask::<ProjectionMask>::whole_aspect(),
        surface_kind: TruthDeltaSurfaceKind::EntityRegion,
    };
}
