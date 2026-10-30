use worth_store_authority::{StoreCurrentAuthorityWitness, StoreDerivedAuthorityEvidence};

fn require_current_authority(_: StoreCurrentAuthorityWitness) {}

fn main() {
    let derived_evidence: StoreDerivedAuthorityEvidence = todo!();

    require_current_authority(derived_evidence);
}
