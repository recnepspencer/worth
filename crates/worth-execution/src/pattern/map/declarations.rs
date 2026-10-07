use super::{MapDenial, MapPartition};
use std::collections::BTreeMap;
use worth_foundational::PartitionIdentity;
use worth_proof::DisjointKeySetFamily;

pub(super) fn check_read_key_order<T, K: Ord>(
    partitions: &[MapPartition<T, K>],
) -> Result<(), MapDenial> {
    for (partition, entry) in partitions.iter().enumerate() {
        for (earlier, pair) in entry.read_keys.windows(2).enumerate() {
            if pair[0] >= pair[1] {
                return Err(MapDenial::ReadKeysNotCanonical {
                    partition,
                    earlier,
                    later: earlier + 1,
                });
            }
        }
    }
    Ok(())
}

pub(super) fn check_read_write_conflicts<T, K: Ord>(
    partitions: &[MapPartition<T, K>],
    write_sets: &DisjointKeySetFamily<PartitionIdentity, K>,
) -> Result<(), MapDenial> {
    let writers: BTreeMap<&K, usize> = write_sets
        .members()
        .iter()
        .enumerate()
        .flat_map(|(writer, (_, keys))| keys.iter().map(move |key| (key, writer)))
        .collect();
    for (reader, partition) in partitions.iter().enumerate() {
        for (read_key, key) in partition.read_keys.iter().enumerate() {
            if let Some(&writer) = writers.get(key) {
                if reader != writer {
                    return Err(MapDenial::ReadWriteConflict {
                        reader: partition.identity,
                        writer: write_sets.members()[writer].0,
                        read_key,
                    });
                }
            }
        }
    }
    Ok(())
}
