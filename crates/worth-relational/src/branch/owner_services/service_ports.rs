use super::owner_binding::RelationalOwnerServiceBinding;
use super::{
    RelationalBranchBasisPort, RelationalBranchLifecyclePort,
    RelationalBranchTransactionAdmissionPort,
};

/// The seven concrete independently borrowable services of one Relational owner.
#[derive(Debug, Clone)]
pub struct RelationalOwnerServicePorts {
    preparation: crate::mvcc::RelationalPreparationPort,
    fork: crate::branch::RelationalForkPort,
    publication: crate::mvcc::RelationalPublicationPort,
    settlement: crate::publication::RelationalSettlementPort,
    basis: RelationalBranchBasisPort,
    lifecycle: RelationalBranchLifecyclePort,
    transaction_admission: RelationalBranchTransactionAdmissionPort,
}

impl RelationalOwnerServicePorts {
    pub fn preparation_port(&self) -> crate::mvcc::RelationalPreparationPort {
        self.preparation.clone()
    }

    pub fn fork_port(&self) -> crate::branch::RelationalForkPort {
        self.fork.clone()
    }

    pub fn publication_port(&self) -> crate::mvcc::RelationalPublicationPort {
        self.publication.clone()
    }

    pub fn settlement_port(&self) -> crate::publication::RelationalSettlementPort {
        self.settlement.clone()
    }

    pub fn basis_port(&self) -> RelationalBranchBasisPort {
        self.basis.clone()
    }

    pub fn lifecycle_port(&self) -> RelationalBranchLifecyclePort {
        self.lifecycle.clone()
    }

    pub fn transaction_admission_port(&self) -> RelationalBranchTransactionAdmissionPort {
        self.transaction_admission.clone()
    }
}

impl crate::runtime::RelationalRuntime {
    /// Issue the complete concrete service bundle for this runtime owner.
    pub fn owner_component_services(&self) -> RelationalOwnerServicePorts {
        let owner = RelationalOwnerServiceBinding::new(self.state_binding(), self.owner_binding());
        RelationalOwnerServicePorts {
            preparation: self.preparation_port(),
            fork: self.fork_port(),
            publication: self.publication_port(),
            settlement: self.settlement_port(),
            basis: RelationalBranchBasisPort::new(owner.clone()),
            lifecycle: RelationalBranchLifecyclePort::new(owner.clone()),
            transaction_admission: RelationalBranchTransactionAdmissionPort::new(owner),
        }
    }
}
