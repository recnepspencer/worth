use worth_foundational::facade::{AspectKey, AspectLocator, LocatorAuthority};
use worth_runtime_bridge::facade::{BridgeCommittedPatchTarget, TruthDeltaSurfaceKind};

fn main() {
    let aspect_locator = AspectLocator::new(
        LocatorAuthority::Authoritative,
        AspectKey::new("profile").expect("valid aspect key"),
    );

    let _target = BridgeCommittedPatchTarget::aspect_surface(
        aspect_locator,
        TruthDeltaSurfaceKind::EntityRegion,
    );
}
