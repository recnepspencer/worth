//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<PhysicalRecoveryPublicationCounters>(
            "workspaces/worth-store/crates/worth-store-recovery-runtime/src/entry/publication.rs",
            "PhysicalRecoveryPublicationCounters",
            128 * 1024,
        );
        owner.field::<u64>("planned_effects");
        owner.field::<u64>("candidate_artifacts_settled");
        owner.field::<u64>("candidate_materializations_performed");
        owner.field::<u64>("candidate_synchronizations_performed");
        owner.field::<u64>("root_protocol_replacements_performed");
        owner.field::<u64>("namespace_synchronizations_performed");
    }
    {
        let _owner = census.owner::<PhysicalRecoveryPublicationSettlement>(
            "workspaces/worth-store/crates/worth-store-recovery-runtime/src/entry/publication.rs",
            "PhysicalRecoveryPublicationSettlement",
            128 * 1024,
        );
    }
    {
        let mut owner = census.owner::<PhysicalRecoveryPublicationSettlementLedger>(
            "workspaces/worth-store/crates/worth-store-recovery-runtime/src/entry/publication.rs",
            "PhysicalRecoveryPublicationSettlementLedger",
            128 * 1024,
        );
        owner.field::<PhysicalRecoveryPublicationSettlement>("settlement");
    }
}
