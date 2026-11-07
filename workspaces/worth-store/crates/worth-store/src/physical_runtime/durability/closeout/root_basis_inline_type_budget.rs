//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<PhysicalRootNamespaceDurabilityEvidence>("workspaces/worth-store/crates/worth-store/src/physical_runtime/durability/closeout/root_basis.rs", "PhysicalRootNamespaceDurabilityEvidence", 128 * 1024);
        owner.field::<RootPublicationCell>("ReopenedCurrentRoot.root");
        owner.field::<PhysicalDurabilityGroupBasis>("PublishedCurrentRoot.group");
        owner.field::<u64>("PublishedCurrentRoot.source_generation");
        owner.field::<u64>("PublishedCurrentRoot.current_generation");
        owner.field::<PhysicalEffectIdentity>("PublishedCurrentRoot.replacement");
        owner.field::<PhysicalEffectIdentity>("PublishedCurrentRoot.namespace_synchronization");
        owner.field::<crate::physical_runtime::PhysicalMutationIdentity>(
            "RetirementCurrentRoot.operation",
        );
        owner.field::<u64>("RetirementCurrentRoot.source_generation");
        owner.field::<u64>("RetirementCurrentRoot.current_generation");
        owner.field::<PhysicalEffectIdentity>("RetirementCurrentRoot.replacement");
        owner.field::<PhysicalEffectIdentity>("RetirementCurrentRoot.namespace_synchronization");
        owner.field::<crate::physical_runtime::PhysicalMutationIdentity>(
            "ManifestResidueCurrentRoot.operation",
        );
        owner.field::<worth_store_physical_format::PersistedRecordIdentity>(
            "ManifestResidueCurrentRoot.manifest",
        );
        owner.field::<u64>("ManifestResidueCurrentRoot.source_generation");
        owner.field::<u64>("ManifestResidueCurrentRoot.current_generation");
        owner.field::<PhysicalEffectIdentity>("ManifestResidueCurrentRoot.replacement");
        owner.field::<PhysicalEffectIdentity>(
            "ManifestResidueCurrentRoot.namespace_synchronization",
        );
        owner.field::<crate::physical_runtime::PhysicalMutationIdentity>(
            "TierEpochCurrentRoot.operation",
        );
        owner.field::<u64>("TierEpochCurrentRoot.epoch");
        owner.field::<u64>("TierEpochCurrentRoot.source_generation");
        owner.field::<u64>("TierEpochCurrentRoot.current_generation");
        owner.field::<PhysicalEffectIdentity>("TierEpochCurrentRoot.replacement");
        owner.field::<PhysicalEffectIdentity>("TierEpochCurrentRoot.namespace_synchronization");
    }
}
