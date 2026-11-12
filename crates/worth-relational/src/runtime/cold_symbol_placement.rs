//! D1634: certification placement changes residency only. Every dictionary
//! member comes from the real native checkpoint exporter. Existing meanings,
//! record allocation, committed rows, refusal and publication outcomes are
//! untouched. Only missing text's real bucket proof becomes demand-loaded.
use crate::durability::{
    data::{RelationalCheckpointManifest, RelationalCheckpointMember},
    medium::{
        RelationalDurableColdSource, RelationalDurableMemberCustody, RelationalDurableMemberLocator,
    },
};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
#[derive(Debug)]
struct Pages {
    members: BTreeMap<RelationalDurableMemberLocator, Arc<[u8]>>,
    reads: Arc<AtomicUsize>,
}
impl RelationalDurableColdSource for Pages {
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        Ok(RelationalDurableMemberCustody::new(()))
    }
    fn read_member(
        &self,
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        if member.length > limit {
            return Err(Cause::LimitExceeded {
                length: member.length,
                limit,
            });
        }
        let bytes = self.members.get(&member).ok_or(Cause::MissingMember)?;
        self.reads.fetch_add(1, Ordering::SeqCst);
        Ok(bytes.to_vec())
    }
    fn read_commit(
        &self,
        _: crate::publication::patch::data::PatchStreamPosition,
        _: u64,
    ) -> Result<Vec<u8>, Cause> {
        Err(Cause::SourceUnavailable)
    }
}
/// Owned observation of actual real page admission; it grants no product power.
pub struct RelationalColdSymbolPlacementForTest {
    reads: Arc<AtomicUsize>,
}
impl RelationalColdSymbolPlacementForTest {
    pub fn page_reads_for_test(&self) -> usize {
        self.reads.load(Ordering::SeqCst)
    }
}
impl super::RelationalRuntime {
    #[doc(hidden)]
    pub fn place_symbol_pages_cold_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> RelationalColdSymbolPlacementForTest {
        let checkpoint = self
            .durability_authority()
            .native_checkpoint(
                execution,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("certification placement exports the genuine dictionary");
        #[derive(serde::Deserialize)]
        struct Input {
            manifest: RelationalCheckpointManifest,
        }
        let input: Input = rmp_serde::from_slice(
            checkpoint
                .bytes()
                .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
                .expect("current native checkpoint frame"),
        )
        .expect("exported manifest");
        let reads = Arc::new(AtomicUsize::new(0));
        let members = checkpoint
            .members()
            .iter()
            .map(|member| match member {
                RelationalCheckpointMember::New { locator, bytes } => {
                    (*locator, Arc::<[u8]>::from(bytes.bytes()))
                }
                RelationalCheckpointMember::Carried { .. } => {
                    panic!("placement starts from a fully resident certification runtime")
                }
            })
            .collect();
        let source = Arc::new(Pages {
            members,
            reads: reads.clone(),
        });
        self.services
            .symbols
            .place_exported_symbol_pages_cold_for_test(&input.manifest, source)
            .expect("real exported symbol pages admit");
        // D1638: a live warm lease may have relied on the fully resident
        // dictionary. Reconstruct only its rebuildable symbol closure through
        // Native's collector, then use real dictionary admission. Never alter
        // its row image, allocate a record or mint an admission mark here.
        crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(|lane| {
            for id in self.history.branch_ids_snapshot() {
                let root = self
                    .history
                    .branch_cell(&id)
                    .and_then(|cell| cell.root())
                    .expect("placement retains each live branch root");
                for region in root.storage_regions() {
                    root.accepted_head_region_witness(region.partition_id)
                        .expect("placement starts resident")
                        .expect("enumerated actual region")
                        .admit_cold_placement_symbols_for_test(
                            lane,
                            execution,
                            &self.services.symbols,
                        )
                        .expect("real branch symbol admission");
                }
            }
            let mirror = self.acquire_partition_edition();
            for (_, region) in mirror.cells() {
                region
                    .accepted_head_lease()
                    .expect("placement starts with a resident mirror")
                    .admit_cold_placement_symbols_for_test(lane, execution, &self.services.symbols)
                    .expect("real mirror symbol admission");
            }
        });
        RelationalColdSymbolPlacementForTest { reads }
    }
}
