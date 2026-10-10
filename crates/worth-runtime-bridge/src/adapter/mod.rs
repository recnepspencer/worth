//! Bridge-owned parent-runtime adapter contracts.

use std::sync::Arc;

use crate::delivery::BridgeDeliveryReceipt;
use crate::error::{
    BridgeDeliveryError, BridgeDeliveryErrorKind, BridgeErrorContext, BridgeLineageSourceError,
    BridgeLineageSourceErrorKind, BridgeMessageError,
};
use crate::routing::BridgeSignalInvalidationDelivery;
use crate::snapshot::{
    AdmittedSnapshotContext, BridgeSnapshotContext, BridgeSnapshotToken, BridgeTruthViewKind,
    MaterializedTruthViewObservation, TruthSnapshotIdentity, TruthSnapshotReader,
};
use crate::source::{
    BridgeSourceCapabilitySet, MaterializedTruthViewPacketSet, PlannedSourceReadPacketSet,
};
use crate::{continuity, input};

mod continuity_lineage;
mod signal_sink;
mod source_materialization;
mod truth_sources;
mod truth_writeback;

#[cfg(test)]
mod tests;

pub use continuity_lineage::{
    BridgeHistoricalLineageAuthority, BridgeHistoricalLineageRequest,
    BridgeHistoricalLineageTopology, BridgeHistoricalResolvedLineageIdentity,
    BridgeHistoricalResolvedRecordIdentity, ContinuityLineageSource,
};
pub use signal_sink::{
    InvalidationSink, SignalBridgeSink, SignalBridgeSinkError, SignalBridgeSinkErrorKind,
};
pub use source_materialization::BridgeSourceAdapter;
mod source_failure;
pub use source_failure::{
    BridgeSourceBindingDenial, RelationalBridgeSourceError, RelationalBridgeSourceErrorTag,
};
pub use truth_sources::{
    CommittedPatchSource, RelationalBridgeSource, RelationalCommittedPatchRequest,
    SnapshotReadSource, SnapshotReaderPool, TruthBranchHeadSource,
};
pub(crate) use truth_writeback::TruthWritebackRequestEvidence;
pub use truth_writeback::{
    TruthWritebackAuthority, TruthWritebackAuthorityError, TruthWritebackReceipt,
    TruthWritebackRequest,
};
