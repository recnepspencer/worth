//! One async behavior per Gate 8.2 exit-proof fault, plus the no-fault
//! success path. Each function is the complete story for its fault: what
//! the ledger learns and what the caller's connection receives.

use std::time::Duration;

use tokio::net::TcpStream;

use crate::protocol::correlation::RailCorrelation;
use crate::protocol::notice::RailRejection;
use crate::protocol::response::RailResponseFrame;
use crate::protocol::wire::write_frame;

use super::completed_effects::CompletedEffects;
use super::ledger::{Ledger, RailReservation};

#[derive(Clone, Copy)]
pub(super) struct CompletionOwners<'a> {
    ledger: &'a Ledger,
    completed_effects: &'a CompletedEffects,
}

impl<'a> CompletionOwners<'a> {
    pub(super) const fn new(ledger: &'a Ledger, completed_effects: &'a CompletedEffects) -> Self {
        Self {
            ledger,
            completed_effects,
        }
    }

    fn complete(self, reservation: RailReservation) {
        self.completed_effects
            .apply_once(
                reservation.correlation().clone(),
                reservation.effect().clone(),
            )
            .expect("a unique rail reservation applies its physical effect exactly once");
        self.ledger.record_completed(&reservation);
    }
}

/// Acknowledge, then complete: the only path that ever reports success.
pub async fn succeed(
    stream: &mut TcpStream,
    reservation: RailReservation,
    owners: CompletionOwners<'_>,
) -> std::io::Result<()> {
    write_frame(stream, &RailResponseFrame::Ack).await?;
    owners.complete(reservation);
    write_frame(stream, &RailResponseFrame::Completed).await
}

/// Commit the effect on the ledger, then lose the response: the connection
/// closes having written nothing at all.
///
/// The ledger write happens before the caller's connection is ever touched,
/// so the effect is real and inspectable through [`report_status`] even
/// though this exchange itself never says so.
pub async fn commit_then_lose_response(reservation: RailReservation, owners: CompletionOwners<'_>) {
    owners.complete(reservation);
}

/// Acknowledge, then never complete: the connection closes cleanly after the
/// acknowledgement.
pub async fn acknowledge_without_completing(
    stream: &mut TcpStream,
    _reservation: RailReservation,
) -> std::io::Result<()> {
    write_frame(stream, &RailResponseFrame::Ack).await
}

/// Acknowledge immediately, then complete only after `delay`. A caller
/// holding a shorter deadline observes a timeout before this arrives; the
/// ledger completes on schedule regardless of whether the caller is still
/// listening.
pub async fn complete_after_delay(
    stream: &mut TcpStream,
    reservation: RailReservation,
    owners: CompletionOwners<'_>,
    delay: Duration,
) -> std::io::Result<()> {
    write_frame(stream, &RailResponseFrame::Ack).await?;
    tokio::time::sleep(delay).await;
    owners.complete(reservation);
    write_frame(stream, &RailResponseFrame::Completed).await
}

/// Acknowledge, then send an explicit duplicate acknowledgement. The attempt
/// never completes.
pub async fn duplicate_acknowledgement(
    stream: &mut TcpStream,
    _reservation: RailReservation,
) -> std::io::Result<()> {
    write_frame(stream, &RailResponseFrame::Ack).await?;
    write_frame(stream, &RailResponseFrame::DuplicateAck).await
}

/// Refuse a payload the rail read and could not serve.
///
/// Written before any ledger admission, so a refused attempt leaves the
/// rail's truth exactly as it was and never becomes an admission.
pub async fn reject(stream: &mut TcpStream, rejection: RailRejection) -> std::io::Result<()> {
    write_frame(stream, &RailResponseFrame::Rejected(rejection)).await
}

/// Report the ledger's own truth for a prior attempt.
pub async fn report_status(
    stream: &mut TcpStream,
    correlation: &RailCorrelation,
    ledger: &Ledger,
) -> std::io::Result<()> {
    let status = ledger.status_of(correlation);
    write_frame(stream, &RailResponseFrame::StatusReport(status)).await
}

/// Report the domain meaning the rail decoded for a prior attempt.
pub async fn report_notice(
    stream: &mut TcpStream,
    correlation: &RailCorrelation,
    ledger: &Ledger,
) -> std::io::Result<()> {
    let notice = ledger.notice_of(correlation);
    write_frame(stream, &RailResponseFrame::NoticeReport(notice)).await
}

pub async fn report_completed_effect_count(
    stream: &mut TcpStream,
    completed_effects: &CompletedEffects,
) -> std::io::Result<()> {
    write_frame(
        stream,
        &RailResponseFrame::CompletedEffectCount(completed_effects.count()),
    )
    .await
}

pub async fn report_completed_notice(
    stream: &mut TcpStream,
    correlation: &RailCorrelation,
    completed_effects: &CompletedEffects,
) -> std::io::Result<()> {
    write_frame(
        stream,
        &RailResponseFrame::CompletedNoticeReport(completed_effects.notice_of(correlation)),
    )
    .await
}
