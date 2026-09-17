//! Wire request carried from a caller to the external rail.

use serde::{Deserialize, Serialize};

use super::correlation::RailCorrelation;
use super::payload::RailEffectPayload;

/// One message sent by a caller over one TCP connection.
///
/// Each connection carries exactly one request and is then closed by the
/// rail, mirroring a real one-shot external call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RailRequest {
    /// Dispatch an external effect attempt.
    ///
    /// The payload is not optional. A rail that could be handed a bare
    /// correlation would have to source the notice's meaning from somewhere
    /// other than the caller that committed it, so "correlation only" is
    /// unrepresentable on this wire rather than merely discouraged.
    Dispatch(RailDispatch),
    /// Ask the rail's own ledger what actually happened to a prior attempt.
    ///
    /// This is the rail's authoritative truth: a caller that lost a response
    /// cannot derive this answer locally and must ask the boundary that owns
    /// it.
    InquireStatus { correlation: RailCorrelation },
    /// Ask the rail which notice it decoded for a prior attempt.
    ///
    /// The answer is the rail's own domain reading of the payload it received,
    /// not an echo of the correlation: a rail that never decoded anything can
    /// only answer `None`.
    InquireNotice { correlation: RailCorrelation },
    /// Ask how many distinct correlations the rail has ever admitted.
    ///
    /// Exactly-once evidence for safe-retry (R8.70): re-dispatch of an already
    /// completed effect must leave this count unchanged.
    InquireAdmissionCount,
    /// Ask how many dispatch frames actually reached this process, including duplicates.
    InquireDispatchContactCount,
    /// Ask how many domain effects the rail physically completed.
    InquireCompletedEffectCount,
    /// Ask which completed domain notice exists for one correlation.
    InquireCompletedNotice { correlation: RailCorrelation },
}

/// One dispatch attempt as it travels the wire.
///
/// Correlation and payload are one value because the rail must bind the
/// idempotency key to the immutable meaning it receives. Test fault selection
/// is deliberately absent from this production request vocabulary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RailDispatch {
    pub correlation: RailCorrelation,
    pub payload: RailEffectPayload,
}
