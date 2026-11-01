//! One selected placement map; ordinary and controlled admission share storage.
use super::{RelationalDurableMemberLocator, StoreColdSource};
use worth_execution::{ExecutionAllocationPolicy, ExecutionOrderedMap, ExecutionOrderedMapDenial};
use worth_store::physical_runtime::{
    AnchoredRecordMemberLocator, AnchoredRecordMembership, AnchoredRecordReadFailure,
    AnchoredRecordReadMembership, AnchoredRecordVersion, AnchoredRecordsDenial,
};

#[derive(Debug)]
pub enum RelationalStoreMemberSelectionDenial {
    Unavailable(AnchoredRecordsDenial),
    Read(AnchoredRecordReadFailure),
    ForeignAnchor,
    Busy,
    DifferentSelection,
    Execution(worth_execution::WorkCeilingDenial),
    Allocation(ExecutionOrderedMapDenial),
}

#[derive(Clone, Copy)]
pub(super) struct Placement {
    pub(super) index: usize,
    pub(super) locator: AnchoredRecordMemberLocator,
}
pub(super) type MemberMap = ExecutionOrderedMap<RelationalDurableMemberLocator, Placement>;

#[derive(Default)]
pub(super) struct SelectedMembers {
    pub(super) version: Option<AnchoredRecordVersion>,
    pub(super) entries: MemberMap,
    pub(super) read: Option<AnchoredRecordReadMembership>,
}

pub(super) fn build(
    members: &[AnchoredRecordMemberLocator],
    mut insert: impl FnMut(
        &mut MemberMap,
        &RelationalDurableMemberLocator,
        &Placement,
    ) -> Result<(), ExecutionOrderedMapDenial>,
) -> Result<MemberMap, RelationalStoreMemberSelectionDenial> {
    let mut result = MemberMap::default();
    for (index, member) in members.iter().enumerate() {
        let key = RelationalDurableMemberLocator {
            sha256: member.sha256(),
            length: member.length(),
        };
        insert(
            &mut result,
            &key,
            &Placement {
                index,
                locator: *member,
            },
        )
        .map_err(RelationalStoreMemberSelectionDenial::Allocation)?;
    }
    Ok(result)
}

impl StoreColdSource {
    /// Existing standalone admission retains its ordinary unbudgeted allocation
    /// contract. It cannot replace a controlled original read or its grants.
    pub(crate) fn select_members(
        &self,
        membership: &AnchoredRecordMembership,
    ) -> Result<(), RelationalStoreMemberSelectionDenial> {
        let version = membership.version();
        {
            let current = self
                .members
                .read()
                .unwrap_or_else(|error| error.into_inner());
            if current.read.is_some() {
                return same_controlled(&current, version);
            }
        }
        let mut built = build(membership.members(), |map, key, value| {
            map.insert_cloned_native_in(key, value, ExecutionAllocationPolicy::SystemAllocation)
        })?;
        // `built` stays outside both guards, including refusal and replacement:
        // disposal of an old metadata map never occurs under owner locks.
        self.custody.select(|| {
            let mut current = self
                .members
                .write()
                .unwrap_or_else(|error| error.into_inner());
            if current.read.is_some() {
                return same_controlled(&current, version);
            }
            if current
                .version
                .is_none_or(|old| version.generation() >= old.generation())
            {
                std::mem::swap(&mut current.entries, &mut built);
                current.version = Some(version);
            }
            Ok(())
        })
    }
}

fn same_controlled(
    current: &SelectedMembers,
    version: AnchoredRecordVersion,
) -> Result<(), RelationalStoreMemberSelectionDenial> {
    if current.version == Some(version) {
        Ok(())
    } else {
        Err(RelationalStoreMemberSelectionDenial::DifferentSelection)
    }
}
