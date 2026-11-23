//! The registered attempt exposes only its lowered naming plan.
use super::WorthQueryPrimaryGraphApplicationAttempt;

impl WorthQueryPrimaryGraphApplicationAttempt {
    /// The naming plan this attempt's transaction attaches, `None` for an
    /// unnamed program; `Err` when declared names were never lowered.
    pub fn naming_plan(
        &self,
    ) -> Result<Option<&worth_relational::facade::lineage::CommitNamingPlan>, ()> {
        if self.effects.has_unlowered_names() {
            return Err(());
        }
        Ok(self.effects.naming_plan())
    }
}
