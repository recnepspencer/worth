//! Type erasure retains the real installed meaning owner for native selection.
use super::{WorthQueryInstalledActionInventory, WorthQueryInstalledApplicationProgram};

mod sealed {
    pub trait InstalledMeaning {}
}
/// Erasure retains the installed meaning owner, rather than copying a runtime action table.
pub trait WorthQueryErasedInstalledProgramMeaning: sealed::InstalledMeaning + Send + Sync {
    fn as_any(&self) -> &dyn std::any::Any;
    fn action_inventory(&self) -> WorthQueryInstalledActionInventory<'_>;
    /// Original installed declarations, including dormant rostered computations.
    fn features(
        &self,
    ) -> &[worth_query_declaration::facade::application_program::ApplicationFeatureDeclaration];
}
impl<Schema: 'static, Program: 'static> sealed::InstalledMeaning
    for WorthQueryInstalledApplicationProgram<Schema, Program>
{
}
impl<Schema: 'static, Program: 'static> WorthQueryErasedInstalledProgramMeaning
    for WorthQueryInstalledApplicationProgram<Schema, Program>
{
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn features(
        &self,
    ) -> &[worth_query_declaration::facade::application_program::ApplicationFeatureDeclaration]
    {
        self.features()
    }
    fn action_inventory(&self) -> WorthQueryInstalledActionInventory<'_> {
        self.action_inventory()
    }
}
