//! Law 1, reach: the product owner control is delegated only here.
use super::WorthQueryProductRuntime;
impl WorthQueryProductRuntime {
    pub(crate) fn operation_control_for_test(
        &self,
    ) -> worth_runtime_world::facade::RuntimeWorldOperationControlForTest {
        self.owner.world_operation_control_for_test()
    }
}
