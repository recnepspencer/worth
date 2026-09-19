//! Wire responses the external rail may send, and the ledger status they can
//! report.

use serde::{Deserialize, Serialize};

use super::notice::{EstateDeathNotice, RailRejection};

/// The rail's own record of what happened to a correlation, independent of
/// whatever bytes the caller did or did not receive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LedgerStatus {
    /// The rail has no record of this correlation: it never began, or it
    /// disappeared before any record was created.
    NoRecord,
    /// The rail acknowledged the attempt but has not completed it.
    Acknowledged,
    /// The rail completed the attempt, whether or not the caller ever
    /// received that news.
    Completed,
}

/// One frame the rail writes to a caller connection.
///
/// `Completed` is written only when the active fault script says the attempt
/// succeeds; no fault path ever writes it. Idempotent re-dispatch of an
/// already-ledgered correlation may replay `Ack`/`Completed` without a new
/// admission (R8.70).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RailResponseFrame {
    Ack,
    DuplicateAck,
    Completed,
    /// The rail read the payload and refused it. Written before any ledger
    /// admission, so a rejected attempt leaves no record and never completes.
    Rejected(RailRejection),
    StatusReport(LedgerStatus),
    /// The notice the rail decoded for a correlation, or `None` if it holds
    /// none.
    NoticeReport(Option<EstateDeathNotice>),
    AdmissionCount(u64),
    DispatchContactCount(u64),
    CompletedEffectCount(u64),
    CompletedNoticeReport(Option<EstateDeathNotice>),
}
