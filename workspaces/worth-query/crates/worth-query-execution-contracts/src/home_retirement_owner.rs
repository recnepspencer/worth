//! Home retirement owner classification and head-cell registration stop.

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorthQueryHomeRetirementOwner {
    ProgramRuntime,
    ConditionalRuntime,
    InstalledResources,
    InstalledProducers,
    InstalledConditionals,
    InboundVerifiers,
    SigningOwners,
    RelationalSource,
    GraphProvider,
    ReceiptBases,
    PublicationReceipts,
    HomeMaintenance,
}

#[derive(Debug)]
pub enum HeadCellRegistrationStop {
    Admission(worth_relational::facade::mvcc::CompanionPreflightStop),
    Native(worth_relational::facade::mvcc::PublicationCompanionRegistrationStop),
    LookupChanged,
}

impl From<worth_relational::facade::mvcc::CompanionPreflightStop> for HeadCellRegistrationStop {
    fn from(stop: worth_relational::facade::mvcc::CompanionPreflightStop) -> Self {
        Self::Admission(stop)
    }
}
