use worth_store_contracts::{
    AcceptedHandoffReadiness, HandoffEvidenceDigestSet, StableDigest, ROADMAP_2_S1_SCOPE,
};
use worth_store_physical_format::{
    InMemoryPhysicalFormatModel, InMemoryPhysicalFormatModelRequest, PersistedPhysicalLayout,
};

fn main() {
    let layout = PersistedPhysicalLayout::builder().build();
    let _reopened = InMemoryPhysicalFormatModel::restore(
        readiness(),
        InMemoryPhysicalFormatModelRequest::physical_format_canonical(),
        layout,
    );
}

fn readiness() -> AcceptedHandoffReadiness {
    AcceptedHandoffReadiness::from_foundational_handoff_artifacts(ROADMAP_2_S1_SCOPE, digest_set())
        .unwrap()
}

fn digest_set() -> HandoffEvidenceDigestSet {
    HandoffEvidenceDigestSet::new(
        digest("backend"),
        digest("deferred"),
        digest("harness"),
        digest("terms"),
        digest("audit"),
        digest("complexity"),
        digest("provenance"),
    )
}

fn digest(name: &str) -> StableDigest {
    StableDigest::new(format!("sha256:{name}")).unwrap()
}
