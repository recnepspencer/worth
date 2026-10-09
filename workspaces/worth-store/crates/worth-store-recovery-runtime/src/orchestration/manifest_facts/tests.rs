//! A routing block spends recovery's manifest bytes: its read is granted
//! what the budget has left, and a block past that is refused with the
//! budget's own counts.

use worth_store::physical_runtime::{ArtifactCeiling, PageAddress, ReadGrant, UnchargedRead};
use worth_store_recovery_physics::PhysicalRootSlotObservation;

use super::{observe_manifest_facts, ManifestObservationBudget};
use crate::entry::{
    PhysicalRecoveryBlockCause, PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDeclaration,
    PhysicalRecoveryLimitDimension,
};
use crate::orchestration::planning::selected_world_fixture::selected_world;
use crate::orchestration::recovery_budget::{recovery_limit_for_test, RecoveryReadBudget};

#[test]
fn a_routing_block_past_what_the_budget_has_left_names_the_whole_budget() {
    selected_world("routing-block-grant", 64).read(|source| {
        let root = source.candidate;
        let reference = root
            .manifest()
            .routing_root()
            .expect("a store holding a record routes it");
        let ceiling = ArtifactCeiling::page(
            source.format,
            PageAddress::RootRoutingBlock {
                generation: reference.generation(),
                block: reference.block(),
            },
        );
        let block = source
            .discovery
            .read(ceiling, ReadGrant::ceiling_only())
            .observed()
            .unwrap();
        let length = block.bytes().expect("the routing root is present").len() as u64;
        // The budget has spent one block's bytes and has one byte less than
        // the routing root's length left.
        let mut values = [1 << 20; 19];
        values[2] = 2 * length - 1;
        let limits = PhysicalRecoveryLimitDeclaration::from_values_for_test(values);
        let mut bytes =
            RecoveryReadBudget::declared(&limits, PhysicalRecoveryLimitDimension::ManifestBytes);
        bytes.charge(&block);
        let (mut remaining_entries, mut blocks_read) = (limits.manifest_entries, 0);
        let Err(failure) = observe_manifest_facts(
            source.discovery,
            &PhysicalRootSlotObservation::Candidate(root.clone()),
            ManifestObservationBudget {
                limits,
                bytes: &mut bytes,
                remaining_entries: &mut remaining_entries,
                blocks_read: &mut blocks_read,
            },
        ) else {
            panic!("a routing block past what is left is refused");
        };
        // The limit counts what was spent beside the grant plus the block's
        // real length, against the whole the budget declared.
        assert_eq!(
            failure.cause(),
            PhysicalRecoveryBlockCause::Limit {
                phase: PhysicalRecoveryBlockKind::MediaObservation,
                limit: recovery_limit_for_test(
                    PhysicalRecoveryLimitDimension::ManifestBytes,
                    2 * length,
                    2 * length - 1,
                )
                .into(),
            }
        );
        // The refused block was neither charged nor counted.
        assert_eq!((bytes.spent(), blocks_read), (length, 0));
    });
}
