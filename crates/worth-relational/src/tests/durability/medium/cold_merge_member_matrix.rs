//! Historical merge planning admits its Full inputs before executing the merge.
use super::super::cold_read_matrix::FAILURES;
use super::super::cold_source::ColdRead;
use super::*;
use crate::storage::data::{RelationalStorageSubject as Storage, RelationalStorageUnavailable};
#[derive(Clone, Copy, Debug)]
enum Subject {
    Head,
    History,
    Symbols,
    Nodes,
    Events,
}
impl Subject {
    const ALL: [Self; 5] = [
        Self::Head,
        Self::History,
        Self::Symbols,
        Self::Nodes,
        Self::Events,
    ];
    fn targets(
        self,
        fixture: &Fixture,
    ) -> std::collections::BTreeSet<RelationalDurableMemberLocator> {
        match self {
            Self::Head | Self::History => fixture
                .manifest
                .roots
                .iter()
                .flat_map(|root| &root.regions)
                .chain(&fixture.manifest.mirror)
                .map(|region| {
                    if matches!(self, Self::Head) {
                        region.head
                    } else {
                        region.history
                    }
                })
                .collect(),
            Self::Symbols => {
                let symbol = fixture.symbol.unwrap();
                fixture
                    .manifest
                    .symbols
                    .iter()
                    .filter(|page| {
                        page.first <= u64::from(symbol.0) && u64::from(symbol.0) <= page.last
                    })
                    .map(|page| page.member)
                    .collect()
            }
            Self::Nodes => fixture
                .manifest
                .lineage
                .iter()
                .map(|page| page.member)
                .collect(),
            Self::Events => fixture
                .manifest
                .lineage_events
                .iter()
                .map(|page| page.member)
                .collect(),
        }
    }
    fn expected(self, fixture: &Fixture, cause: Cause) -> Option<MergeExecutionPreparationError> {
        let subject = match self {
            Self::Head | Self::History => Storage::Region(PartitionId::main()),
            Self::Symbols => {
                let symbol = fixture.symbol.unwrap();
                let page = fixture
                    .manifest
                    .symbols
                    .iter()
                    .find(|page| {
                        page.first <= u64::from(symbol.0) && u64::from(symbol.0) <= page.last
                    })
                    .unwrap();
                Storage::Symbols {
                    first: crate::symbols::data::Symbol(page.first.try_into().unwrap()),
                    last: crate::symbols::data::Symbol(page.last.try_into().unwrap()),
                }
            }
            Self::Nodes | Self::Events => return None,
        };
        Some(MergeExecutionPreparationError::Planning(
            MergePlanningError::Execution(RelationalExecutionDenialCause::StorageUnavailable(
                RelationalStorageUnavailable { subject, cause },
            )),
        ))
    }
}

#[cfg(test)]
#[path = "cold_merge_member_matrix/exhaustive.rs"]
mod exhaustive;
