//! Effect-only execution of a complete partition-publication plan.

use crate::storage::overlay::mirror_merge::{
    merge_entity_slots, publish_hybrid_partition, publish_whole_partition,
};
use crate::storage::overlay::{summarize_entity_chunk_plan, EntityWorkingSetLayout};

use crate::performance::data::runtime_complexity_counters::RuntimeComplexityCounters;
use crate::runtime::state::subsystems::storage::{PartitionEditionCopyLane, StorageSubsystem};
use crate::storage::mirror_plan::{
    PartitionPublicationStrategy, PlannedPartitionPublication, PreparedMirrorPublication,
};
use std::sync::Mutex;

pub(crate) fn publish_prepared_mirror(
    partitions: &StorageSubsystem,
    counters: &Mutex<RuntimeComplexityCounters>,
    plan: PreparedMirrorPublication,
) {
    for partition in plan.partitions {
        execute_partition(partitions, counters, partition);
    }
}

fn execute_partition(
    partitions: &StorageSubsystem,
    counters: &Mutex<RuntimeComplexityCounters>,
    publication: PlannedPartitionPublication,
) {
    let mut writer = partitions.edit(counters, PartitionEditionCopyLane::Ordinary);
    match publication {
        PlannedPartitionPublication::New {
            target,
            entity_layout,
            entity_slots,
        } => {
            if let Some(layout) = entity_layout {
                record_entity_replacement(counters, entity_slots, layout);
            }
            writer.install_target(target);
        }
        PlannedPartitionPublication::Existing(mut publication) => {
            let target = publication.target.clone();
            let mut entity_chunks = None;
            let readmission = publication.readmission.clone();
            writer.transform_target(&target, readmission, |mut base| {
                match publication.strategy {
                    PartitionPublicationStrategy::EntityOnly(layout) => {
                        entity_chunks = merge_entity_slots(&mut base, &mut publication, layout);
                        base
                    }
                    PartitionPublicationStrategy::HybridGraph { entity_layout } => {
                        let (merged, chunks) =
                            publish_hybrid_partition(publication, base, entity_layout);
                        count_published_chunks(counters, chunks);
                        merged
                    }
                    PartitionPublicationStrategy::WholePartition => {
                        publish_whole_partition(publication, base)
                    }
                }
            });
            drop(writer);
            count_published_chunks(counters, entity_chunks);
        }
    }
}

fn count_published_chunks(
    counters: &Mutex<RuntimeComplexityCounters>,
    published_chunks: Option<usize>,
) {
    if let Some(published_chunks) = published_chunks {
        counters
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .aosoa_entity_chunks_published += published_chunks;
    }
}

fn record_entity_replacement(
    counters: &Mutex<RuntimeComplexityCounters>,
    entity_slots: usize,
    layout: EntityWorkingSetLayout,
) {
    if matches!(layout, EntityWorkingSetLayout::AoSoACandidate { .. }) {
        let summary = summarize_entity_chunk_plan(entity_slots, layout);
        let mut counters = counters.lock().unwrap_or_else(|error| error.into_inner());
        counters.aosoa_entity_chunks_published += summary.chunk_count.max(1);
        counters.aosoa_entity_slot_soa_merges += entity_slots;
        counters.aosoa_publish_soa_merge_count += 1;
    }
}
