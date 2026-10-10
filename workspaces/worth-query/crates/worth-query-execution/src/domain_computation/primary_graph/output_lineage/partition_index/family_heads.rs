//! The head each partition of one output source offers a reader.

use super::*;
use worth_relational::facade::mvcc::CompanionPreflightStop;

pub(in crate::domain_computation::primary_graph::output_lineage) struct FamilyPublicationHead<'a> {
    pub(in crate::domain_computation::primary_graph::output_lineage) coordinate: ProductCoordinate,
    pub(in crate::domain_computation::primary_graph::output_lineage) slot: usize,
    pub(in crate::domain_computation::primary_graph::output_lineage) recorded: &'a RecordedOutput,
}

impl WorthQueryApplicationOutputLineage {
    /// The latest published output of every partition at or below
    /// `coordinate` in its own occurrence. A prepared vacancy is passed, and
    /// every visited locator is one unit of work; a partition with no locator
    /// at or below the coordinate still costs its lookup.
    #[allow(clippy::type_complexity)]
    pub(in crate::domain_computation::primary_graph::output_lineage) fn family_partition_heads_budgeted(
        &self,
        source: &SemanticSource,
        coordinate: ProductCoordinate,
        maximum_work: usize,
    ) -> Result<(Vec<(Option<[u8; 32]>, FamilyPublicationHead<'_>)>, usize), CompanionPreflightStop>
    {
        if maximum_work == 0 {
            return Err(CompanionPreflightStop::WorkExhausted {
                required: 1,
                maximum: 0,
            });
        }
        let mut heads = Vec::new();
        let mut work = 0usize;
        let charge = |work: &mut usize| {
            *work = work
                .checked_add(1)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
            if *work > maximum_work {
                return Err(CompanionPreflightStop::WorkExhausted {
                    required: u64::try_from(*work)
                        .map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?,
                    maximum: u64::try_from(maximum_work)
                        .map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?,
                });
            }
            Ok(())
        };
        if let Some(partitions) = self
            .partition_index
            .slots
            .get(source)
            .and_then(|occurrences| occurrences.get(&coordinate.occurrence))
        {
            for (partition, generations) in partitions {
                let mut visited = false;
                for (generation, slot) in generations.range(..=coordinate.generation).rev() {
                    visited = true;
                    charge(&mut work)?;
                    let Some(slot) = slot.get() else { continue };
                    heads.push((
                        *partition,
                        FamilyPublicationHead {
                            coordinate: ProductCoordinate {
                                occurrence: coordinate.occurrence,
                                generation: *generation,
                            },
                            slot: *slot,
                            recorded: self.recorded_at_partition_slot(
                                source,
                                coordinate.occurrence,
                                *generation,
                                *partition,
                                *slot,
                            ),
                        },
                    ));
                    break;
                }
                if !visited {
                    charge(&mut work)?;
                }
            }
        }
        Ok((heads, work.max(1)))
    }
}
