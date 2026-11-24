//! Effect-only execution of a complete partition-publication plan.

use crate::storage::overlay::mirror_merge::{
    merge_entity_slots, publish_hybrid_partition, publish_whole_partition,
};
use crate::storage::overlay::{summarize_entity_chunk_plan, EntityWorkingSetLayout};

use super::super::StorageAuthority;
use crate::storage::mirror_plan::{
    PartitionPublicationStrategy, PlannedPartitionPublication, PreparedMirrorPublication,
};

pub(super) fn execute_partition_publication(
    authority: &StorageAuthority<'_>,
    plan: PreparedMirrorPublication,
) {
    for partition in plan.partitions {
        execute_partition(authority, partition);
    }
}

fn execute_partition(authority: &StorageAuthority<'_>, publication: PlannedPartitionPublication) {
    let mut writer = authority.runtime.edit_partitions();
    match publication {
        PlannedPartitionPublication::New {
            target,
            entity_layout,
            entity_slots,
        } => {
            if let Some(layout) = entity_layout {
                record_entity_replacement(authority.runtime, entity_slots, layout);
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
                        count_published_chunks(authority.runtime, chunks);
                        merged
                    }
                    PartitionPublicationStrategy::WholePartition => {
                        publish_whole_partition(publication, base)
                    }
                }
            });
            drop(writer);
            count_published_chunks(authority.runtime, entity_chunks);
        }
    }
}

fn count_published_chunks(
    runtime: &crate::runtime::RelationalRuntime,
    published_chunks: Option<usize>,
) {
    if let Some(published_chunks) = published_chunks {
        runtime
            .performance_access()
            .count_aosoa_publish_chunks(published_chunks);
    }
}

fn record_entity_replacement(
    runtime: &crate::runtime::RelationalRuntime,
    entity_slots: usize,
    layout: EntityWorkingSetLayout,
) {
    if matches!(layout, EntityWorkingSetLayout::AoSoACandidate { .. }) {
        let summary = summarize_entity_chunk_plan(entity_slots, layout);
        runtime
            .performance_access()
            .count_aosoa_publish_soa_merge(summary.chunk_count.max(1), entity_slots);
    }
}
