//! Law 1, reach: the product owner control is delegated only here.
use super::WorthQueryProductRuntime;
impl WorthQueryProductRuntime {
    pub fn operation_control_for_test(
        &self,
    ) -> worth_runtime_world::facade::RuntimeWorldOperationControlForTest {
        self.owner
            .publication_port()
            .world_operation_control_for_test()
            .expect("the installed World admits its actual operation control")
    }
}

impl WorthQueryProductRuntime {
    /// Observe the installed World issuer without consuming a coordinate.
    pub fn public_branch_name_frontier_for_test(
        &self,
    ) -> Result<u64, worth_runtime_world::facade::RuntimeWorldOwnerUnavailable> {
        self.owner
            .publication_port()
            .creation_name_frontier_for_test()
    }
}
