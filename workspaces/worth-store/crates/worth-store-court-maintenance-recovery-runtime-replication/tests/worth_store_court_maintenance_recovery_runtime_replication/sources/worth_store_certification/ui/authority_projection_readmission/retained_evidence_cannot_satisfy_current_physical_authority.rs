use worth_store_authority::{
    StoreCurrentPhysicalAuthorityWitness, StoreRetainedAuthorityEvidence,
};

fn require_current_physical_authority(_: StoreCurrentPhysicalAuthorityWitness<'_>) {}

fn main() {
    let retained_evidence: StoreRetainedAuthorityEvidence = todo!();
    require_current_physical_authority(retained_evidence);
}
