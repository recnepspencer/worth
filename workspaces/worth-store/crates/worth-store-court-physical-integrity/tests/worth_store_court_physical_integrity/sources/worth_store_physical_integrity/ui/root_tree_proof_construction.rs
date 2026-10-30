use worth_store_physical_integrity::{
    IntegrityValidatedBootstrapCatalog, IntegrityValidatedRootRoutingBlock,
    IntegrityValidatedSegmentMembershipBlock, PhysicalArtifactScope,
};

fn forge_bootstrap(scope: PhysicalArtifactScope) {
    let _forged = IntegrityValidatedBootstrapCatalog { scope };
}

fn forge_root_routing(scope: PhysicalArtifactScope) {
    let _forged = IntegrityValidatedRootRoutingBlock { scope };
}

fn forge_segment_membership(scope: PhysicalArtifactScope) {
    let _forged = IntegrityValidatedSegmentMembershipBlock { scope };
}

fn main() {}
