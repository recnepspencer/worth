//! Law 1, reach: the product owner control is delegated only here.
use super::WorthQueryProductRuntime;
impl WorthQueryProductRuntime {
    pub(crate) fn operation_control(
        &self,
    ) -> worth_runtime_world::facade::RuntimeWorldOperationControl {
        self.owner.world_operation_control_for_test()
    }
}
