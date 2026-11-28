use super::{LineageRecord, LineageRecordKind};

use crate::state::SignalBranchId;

impl LineageRecord {
    pub fn branch_fork(
        sequence: u64,
        emitted_on_branch_id: SignalBranchId,
        created_branch_id: SignalBranchId,
        parent_branch_id: SignalBranchId,
        created_branch_display_name: impl Into<String>,
        parent_branch_display_name: impl Into<String>,
    ) -> Self {
        Self::new(
            sequence,
            emitted_on_branch_id,
            LineageRecordKind::BranchFork {
                created_branch_id,
                parent_branch_id,
                created_branch_display_name: created_branch_display_name.into(),
                parent_branch_display_name: parent_branch_display_name.into(),
            },
        )
    }

    pub fn branch_switch(
        sequence: u64,
        emitted_on_branch_id: SignalBranchId,
        from_branch_id: SignalBranchId,
        to_branch_id: SignalBranchId,
        from_branch_display_name: impl Into<String>,
        to_branch_display_name: impl Into<String>,
    ) -> Self {
        Self::new(
            sequence,
            emitted_on_branch_id,
            LineageRecordKind::BranchSwitch {
                from_branch_id,
                to_branch_id,
                from_branch_display_name: from_branch_display_name.into(),
                to_branch_display_name: to_branch_display_name.into(),
            },
        )
    }
}
