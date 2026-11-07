//! A closed court replaces equivalent epoch identity through the real owner.
use super::RelationalRuntime;
use crate::mvcc::RelationalPublicationDenial;
use std::sync::Arc;

impl RelationalRuntime {
    /// Replace only the current equivalent outer pair. No schema, generation,
    /// callback, root or authority can be supplied or returned by this control.
    #[doc(hidden)]
    pub fn replace_equivalent_configuration_epoch_for_test(
        &mut self,
    ) -> Result<(), RelationalPublicationDenial> {
        let owner = self.owner_binding();
        let runtime_instance_id = self.runtime_instance_id();
        let unavailable = || RelationalPublicationDenial::OwnerUnavailable {
            runtime_instance_id,
        };
        if !self.is_owner() {
            return Err(unavailable());
        }
        let Some(_operation) = owner.admit() else {
            return Err(unavailable());
        };
        self.reconfigure(|configuration| {
            let binding = configuration.binding();
            let mut pair = binding.initial_installation();
            let config = Arc::clone(&pair.config);
            let contracts = Arc::new(pair.schema_contract_runtime.as_ref().clone());
            pair.config = config;
            pair.schema_contract_runtime = contracts;
        });
        Ok(())
    }
}
