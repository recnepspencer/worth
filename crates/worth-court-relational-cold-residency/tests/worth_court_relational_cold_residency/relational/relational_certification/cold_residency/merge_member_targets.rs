//! Select only actual emitted merge operands for each named physical fault.
use super::merge_source::MergeFixture;
use worth_relational::facade::{
    durability::RelationalDurableMemberLocator,
    identity::PartitionId,
    merge::{MergeExecutionPreparationError, MergePlanningError},
    storage::{
        RelationalStorageSubject as Storage, RelationalStorageUnavailable,
        RelationalStorageUnavailableCause as Cause,
    },
    transactions::RelationalExecutionDenialCause,
};
#[derive(Clone, Copy, Debug)]
pub(super) enum Subject {
    Head,
    History,
    Symbols,
    Nodes,
    Events,
}
impl Subject {
    pub(super) const ALL: [Self; 5] = [
        Self::Head,
        Self::History,
        Self::Symbols,
        Self::Nodes,
        Self::Events,
    ];
    pub(super) fn targets(
        self,
        fixture: &MergeFixture,
    ) -> std::collections::BTreeSet<RelationalDurableMemberLocator> {
        match self {
            Self::Head | Self::History => fixture
                .directory
                .roots
                .iter()
                .flat_map(|root| &root.regions)
                .chain(&fixture.directory.mirror)
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
                    .directory
                    .symbols
                    .iter()
                    .filter(|page| {
                        page.first <= u64::from(symbol.0) && u64::from(symbol.0) <= page.last
                    })
                    .map(|page| page.member)
                    .collect()
            }
            Self::Nodes => fixture
                .directory
                .lineage
                .iter()
                .map(|page| page.member)
                .collect(),
            Self::Events => fixture
                .directory
                .lineage_events
                .iter()
                .map(|page| page.member)
                .collect(),
        }
    }
    pub(super) fn expected(
        self,
        fixture: &MergeFixture,
        cause: Cause,
    ) -> Option<MergeExecutionPreparationError> {
        let subject = match self {
            Self::Head | Self::History => Storage::Region(PartitionId::main()),
            Self::Symbols => {
                let symbol = fixture.symbol.unwrap();
                let page = fixture
                    .directory
                    .symbols
                    .iter()
                    .find(|page| {
                        page.first <= u64::from(symbol.0) && u64::from(symbol.0) <= page.last
                    })
                    .unwrap();
                Storage::Symbols {
                    first: worth_relational::facade::symbols::Symbol(
                        page.first.try_into().unwrap(),
                    ),
                    last: worth_relational::facade::symbols::Symbol(page.last.try_into().unwrap()),
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
