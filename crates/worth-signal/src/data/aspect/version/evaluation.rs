//! Apply scoped versions through the selected partition's ordered range.
use super::{AspectVersion, BTreeMap, ChangedRegion, PartitionSubscription, PartitionToken};

pub(super) fn apply(
    partitions: &mut BTreeMap<PartitionToken, AspectVersion>,
    details: &mut BTreeMap<PartitionSubscription, AspectVersion>,
    version: AspectVersion,
    changed_regions: &[ChangedRegion],
) {
    for region in changed_regions {
        partitions.insert(region.partition.clone(), version);
        if let Some(detail) = &region.detail {
            details.insert(
                PartitionSubscription::partition_and_detail(
                    region.partition.clone(),
                    detail.clone(),
                ),
                version,
            );
        } else {
            // Partition is the first ordering axis and None is the lowest detail.
            let lower = PartitionSubscription::whole_partition(region.partition.clone());
            for (scope, scoped_version) in details.range_mut(lower..) {
                if scope.partition != region.partition {
                    break;
                }
                *scoped_version = version;
            }
        }
    }
}
