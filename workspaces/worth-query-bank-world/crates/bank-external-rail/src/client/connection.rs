//! A fresh TCP connection to the external rail for one exchange.
//!
//! Every dispatch and every status inquiry opens its own connection,
//! mirroring the rail's one-shot-per-connection protocol.

use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::TcpStream;

use crate::protocol::correlation::RailCorrelation;
use crate::protocol::notice::EstateDeathNotice;
use crate::protocol::request::{RailDispatch, RailRequest};
use crate::protocol::response::{LedgerStatus, RailResponseFrame};
use crate::protocol::wire::{read_frame, write_frame, FrameRead};

use super::outcome::RailExchangeOutcome;

/// A transport-level failure to reach the rail or read its answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailTransportFailure {
    Disconnected,
    TimedOut,
}

/// Connects to the rail, sends a production dispatch request, and observes
/// the outcome within `frame_timeout` per frame.
///
/// The rail's protocol always writes `Ack` first when it writes anything at
/// all: `CommitThenLoseResponse` and `DisappearMidDispatch` write nothing.
/// `CompleteAfterDelay` faults use a wall-clock delay that a short
/// `frame_timeout` proves is missed, and a later [`inquire_status`] call
/// proves the rail completed anyway.
pub async fn dispatch(
    addr: SocketAddr,
    attempt: RailDispatch,
    frame_timeout: Duration,
) -> RailExchangeOutcome {
    let mut stream = match TcpStream::connect(addr).await {
        Ok(stream) => stream,
        Err(_) => return RailExchangeOutcome::Disconnected,
    };
    let request = RailRequest::Dispatch(attempt);
    if write_frame(&mut stream, &request).await.is_err() {
        return RailExchangeOutcome::Disconnected;
    }

    match read_response_frame(&mut stream, frame_timeout).await {
        Err(failure) => failure.into(),
        Ok(RailResponseFrame::Rejected(rejection)) => RailExchangeOutcome::Rejected(rejection),
        Ok(RailResponseFrame::Ack) => match read_response_frame(&mut stream, frame_timeout).await {
            Ok(RailResponseFrame::Completed) => RailExchangeOutcome::Completed,
            Ok(RailResponseFrame::DuplicateAck) => RailExchangeOutcome::DuplicateAcknowledgement,
            Err(RailTransportFailure::Disconnected) => RailExchangeOutcome::Acknowledged,
            Err(RailTransportFailure::TimedOut) => RailExchangeOutcome::TimedOut,
            Ok(
                RailResponseFrame::Ack
                | RailResponseFrame::Rejected(_)
                | RailResponseFrame::StatusReport(_)
                | RailResponseFrame::NoticeReport(_)
                | RailResponseFrame::AdmissionCount(_)
                | RailResponseFrame::DispatchContactCount(_)
                | RailResponseFrame::CompletedEffectCount(_)
                | RailResponseFrame::CompletedNoticeReport(_),
            ) => unreachable!(
                "the rail never sends a second Ack, a rejection, or a report after Dispatch's Ack"
            ),
        },
        Ok(
            RailResponseFrame::Completed
            | RailResponseFrame::DuplicateAck
            | RailResponseFrame::StatusReport(_)
            | RailResponseFrame::NoticeReport(_)
            | RailResponseFrame::AdmissionCount(_)
            | RailResponseFrame::DispatchContactCount(_)
            | RailResponseFrame::CompletedEffectCount(_)
            | RailResponseFrame::CompletedNoticeReport(_),
        ) => unreachable!(
            "the rail's first Dispatch frame is always Ack or Rejected when it writes anything"
        ),
    }
}

/// Asks the rail which notice it decoded for `correlation`.
///
/// This reads the rail's own domain understanding, not an echo: a rail that
/// received a correlation but no decodable payload answers `None`.
pub async fn inquire_notice(
    addr: SocketAddr,
    correlation: RailCorrelation,
    frame_timeout: Duration,
) -> Result<Option<EstateDeathNotice>, RailTransportFailure> {
    let mut stream = TcpStream::connect(addr)
        .await
        .map_err(|_| RailTransportFailure::Disconnected)?;
    let request = RailRequest::InquireNotice { correlation };
    write_frame(&mut stream, &request)
        .await
        .map_err(|_| RailTransportFailure::Disconnected)?;
    match read_response_frame(&mut stream, frame_timeout).await? {
        RailResponseFrame::NoticeReport(notice) => Ok(notice),
        RailResponseFrame::Ack
        | RailResponseFrame::DuplicateAck
        | RailResponseFrame::Completed
        | RailResponseFrame::Rejected(_)
        | RailResponseFrame::StatusReport(_)
        | RailResponseFrame::AdmissionCount(_)
        | RailResponseFrame::DispatchContactCount(_)
        | RailResponseFrame::CompletedEffectCount(_)
        | RailResponseFrame::CompletedNoticeReport(_) => {
            unreachable!("the rail only ever answers InquireNotice with a NoticeReport frame")
        }
    }
}

/// Asks the rail's own ledger what actually happened to `correlation`.
pub async fn inquire_status(
    addr: SocketAddr,
    correlation: RailCorrelation,
    frame_timeout: Duration,
) -> Result<LedgerStatus, RailTransportFailure> {
    let mut stream = TcpStream::connect(addr)
        .await
        .map_err(|_| RailTransportFailure::Disconnected)?;
    let request = RailRequest::InquireStatus { correlation };
    write_frame(&mut stream, &request)
        .await
        .map_err(|_| RailTransportFailure::Disconnected)?;
    match read_response_frame(&mut stream, frame_timeout).await? {
        RailResponseFrame::StatusReport(status) => Ok(status),
        RailResponseFrame::Ack
        | RailResponseFrame::DuplicateAck
        | RailResponseFrame::Completed
        | RailResponseFrame::Rejected(_)
        | RailResponseFrame::NoticeReport(_)
        | RailResponseFrame::AdmissionCount(_)
        | RailResponseFrame::DispatchContactCount(_)
        | RailResponseFrame::CompletedEffectCount(_)
        | RailResponseFrame::CompletedNoticeReport(_) => {
            unreachable!("the rail only ever answers InquireStatus with a StatusReport frame")
        }
    }
}

/// Asks how many distinct correlations the rail has ever admitted to its ledger.
pub async fn inquire_admission_count(
    addr: SocketAddr,
    frame_timeout: Duration,
) -> Result<u64, RailTransportFailure> {
    let mut stream = TcpStream::connect(addr)
        .await
        .map_err(|_| RailTransportFailure::Disconnected)?;
    let request = RailRequest::InquireAdmissionCount;
    write_frame(&mut stream, &request)
        .await
        .map_err(|_| RailTransportFailure::Disconnected)?;
    match read_response_frame(&mut stream, frame_timeout).await? {
        RailResponseFrame::AdmissionCount(count) => Ok(count),
        RailResponseFrame::Ack
        | RailResponseFrame::DuplicateAck
        | RailResponseFrame::Completed
        | RailResponseFrame::Rejected(_)
        | RailResponseFrame::StatusReport(_)
        | RailResponseFrame::NoticeReport(_)
        | RailResponseFrame::DispatchContactCount(_)
        | RailResponseFrame::CompletedEffectCount(_)
        | RailResponseFrame::CompletedNoticeReport(_) => {
            unreachable!("the rail only ever answers InquireAdmissionCount with AdmissionCount")
        }
    }
}

/// Asks the rail how many dispatch frames crossed its TCP boundary, including duplicates.
pub async fn inquire_dispatch_contact_count(
    addr: SocketAddr,
    frame_timeout: Duration,
) -> Result<u64, RailTransportFailure> {
    let mut stream = connect_and_write(addr, RailRequest::InquireDispatchContactCount).await?;
    match read_response_frame(&mut stream, frame_timeout).await? {
        RailResponseFrame::DispatchContactCount(count) => Ok(count),
        RailResponseFrame::Ack
        | RailResponseFrame::DuplicateAck
        | RailResponseFrame::Completed
        | RailResponseFrame::Rejected(_)
        | RailResponseFrame::StatusReport(_)
        | RailResponseFrame::NoticeReport(_)
        | RailResponseFrame::AdmissionCount(_)
        | RailResponseFrame::CompletedEffectCount(_)
        | RailResponseFrame::CompletedNoticeReport(_) => {
            unreachable!("the rail only answers InquireDispatchContactCount with its exact count")
        }
    }
}

/// Asks how many physical domain consequences the rail completed.
pub async fn inquire_completed_effect_count(
    addr: SocketAddr,
    frame_timeout: Duration,
) -> Result<u64, RailTransportFailure> {
    let mut stream = connect_and_write(addr, RailRequest::InquireCompletedEffectCount).await?;
    match read_response_frame(&mut stream, frame_timeout).await? {
        RailResponseFrame::CompletedEffectCount(count) => Ok(count),
        RailResponseFrame::Ack
        | RailResponseFrame::DuplicateAck
        | RailResponseFrame::Completed
        | RailResponseFrame::Rejected(_)
        | RailResponseFrame::StatusReport(_)
        | RailResponseFrame::NoticeReport(_)
        | RailResponseFrame::AdmissionCount(_)
        | RailResponseFrame::DispatchContactCount(_)
        | RailResponseFrame::CompletedNoticeReport(_) => unreachable!(
            "the rail only ever answers InquireCompletedEffectCount with its exact count"
        ),
    }
}

/// Asks the independent consequence owner which notice was physically applied.
pub async fn inquire_completed_notice(
    addr: SocketAddr,
    correlation: RailCorrelation,
    frame_timeout: Duration,
) -> Result<Option<EstateDeathNotice>, RailTransportFailure> {
    let mut stream =
        connect_and_write(addr, RailRequest::InquireCompletedNotice { correlation }).await?;
    match read_response_frame(&mut stream, frame_timeout).await? {
        RailResponseFrame::CompletedNoticeReport(notice) => Ok(notice),
        RailResponseFrame::Ack
        | RailResponseFrame::DuplicateAck
        | RailResponseFrame::Completed
        | RailResponseFrame::Rejected(_)
        | RailResponseFrame::StatusReport(_)
        | RailResponseFrame::NoticeReport(_)
        | RailResponseFrame::AdmissionCount(_)
        | RailResponseFrame::DispatchContactCount(_)
        | RailResponseFrame::CompletedEffectCount(_) => {
            unreachable!("the rail only ever answers InquireCompletedNotice with its consequence")
        }
    }
}

async fn connect_and_write(
    addr: SocketAddr,
    request: RailRequest,
) -> Result<TcpStream, RailTransportFailure> {
    let mut stream = TcpStream::connect(addr)
        .await
        .map_err(|_| RailTransportFailure::Disconnected)?;
    write_frame(&mut stream, &request)
        .await
        .map_err(|_| RailTransportFailure::Disconnected)?;
    Ok(stream)
}

async fn read_response_frame(
    stream: &mut TcpStream,
    frame_timeout: Duration,
) -> Result<RailResponseFrame, RailTransportFailure> {
    match tokio::time::timeout(frame_timeout, read_frame::<_, RailResponseFrame>(stream)).await {
        Err(_) => Err(RailTransportFailure::TimedOut),
        Ok(Err(_)) => Err(RailTransportFailure::Disconnected),
        Ok(Ok(FrameRead::Disconnected)) => Err(RailTransportFailure::Disconnected),
        Ok(Ok(FrameRead::Frame(frame))) => Ok(frame),
    }
}

impl From<RailTransportFailure> for RailExchangeOutcome {
    fn from(failure: RailTransportFailure) -> Self {
        match failure {
            RailTransportFailure::Disconnected => RailExchangeOutcome::Disconnected,
            RailTransportFailure::TimedOut => RailExchangeOutcome::TimedOut,
        }
    }
}
