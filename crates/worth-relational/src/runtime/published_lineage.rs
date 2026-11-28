#[cfg(test)]
use crate::runtime::state::subsystems::lineage::LineageSubsystem;
use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    #[cfg(test)]
    pub(crate) fn lineage_authority(&self) -> &LineageSubsystem {
        &self.lineage
    }
}
