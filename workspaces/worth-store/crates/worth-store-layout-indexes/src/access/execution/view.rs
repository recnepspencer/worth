#[derive(Debug, PartialEq, Eq)]
pub enum ExecutedLayoutOperation {
    LsmLookup(Box<crate::BaselineLsmLookupExecution>),
    LsmRunPublication(Box<crate::BaselineLsmManifestPublicationExecution>),
    LsmReplay(Box<crate::BaselineLsmReplayExecution>),
    LsmCompaction(Box<crate::BaselineLsmCompactionPublicationReceipt>),
    DegradedScan(Box<super::DegradedScanExecution>),
}

macro_rules! observe_owner_execution {
    ($owner:ty, $case:ident) => {
        impl From<$owner> for ExecutedLayoutOperation {
            fn from(executed: $owner) -> Self {
                Self::$case(Box::new(executed))
            }
        }
    };
}

observe_owner_execution!(crate::BaselineLsmLookupExecution, LsmLookup);
observe_owner_execution!(
    crate::BaselineLsmManifestPublicationExecution,
    LsmRunPublication
);
observe_owner_execution!(crate::BaselineLsmReplayExecution, LsmReplay);
observe_owner_execution!(
    crate::BaselineLsmCompactionPublicationReceipt,
    LsmCompaction
);
observe_owner_execution!(super::DegradedScanExecution, DegradedScan);
