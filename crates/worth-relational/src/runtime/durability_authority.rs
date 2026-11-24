use crate::durability::authority::recovered_runtime::DurabilityRecoveryAuthority;
use crate::durability::authority::runtime_access::DurabilityAuthority;
use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    pub fn durability_authority(&self) -> DurabilityAuthority<'_> {
        DurabilityAuthority::new(self)
    }

    pub fn durability_recovery(&mut self) -> DurabilityRecoveryAuthority<'_> {
        DurabilityRecoveryAuthority::new(self)
    }
}
