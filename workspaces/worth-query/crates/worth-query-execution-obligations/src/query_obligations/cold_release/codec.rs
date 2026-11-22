//! Version-two cold release payload: subject family and position, fence and
//! terminal. No capability or process-local identity is encoded.
use super::super::codec::Reader;
use super::*;
use std::num::NonZeroU64;

pub(in crate::query_obligations) fn encode(out: &mut Vec<u8>, release: &QueryColdRelease) {
    let (family, position) = match release.subject {
        QueryColdReleaseSubject::ManagedRun(identity) => (0, identity.acquire_position()),
        QueryColdReleaseSubject::ManagedCleanup(identity) => (1, identity.acquire_position()),
        QueryColdReleaseSubject::Resource(identity) => (2, identity.acquire_position()),
    };
    out.push(family);
    out.extend_from_slice(&position.get().to_le_bytes());
    out.extend_from_slice(&release.fence.to_le_bytes());
    out.push(match release.terminal {
        QueryColdReleaseTerminal::Abandoned => 0,
        QueryColdReleaseTerminal::ReleaseOutstanding => 1,
    });
}

pub(in crate::query_obligations) fn decode(
    reader: &mut Reader<'_>,
    home: HomeIdentity,
) -> Result<QueryColdRelease, Denial> {
    let family = reader.take::<1>()?[0];
    let position =
        NonZeroU64::new(u64::from_le_bytes(reader.take()?)).ok_or(Denial::MalformedFrame)?;
    let subject = match family {
        0 => QueryColdReleaseSubject::ManagedRun(ManagedRunObligationIdentity::from_recorded(
            home, position,
        )),
        1 => QueryColdReleaseSubject::ManagedCleanup(
            ManagedCleanupObligationIdentity::from_recorded(home, position),
        ),
        2 => QueryColdReleaseSubject::Resource(OwnedAsyncObligationIdentity::from_recorded(
            home, position,
        )),
        _ => return Err(Denial::MalformedFrame),
    };
    let fence = u64::from_le_bytes(reader.take()?);
    let terminal = match reader.take::<1>()?[0] {
        0 => QueryColdReleaseTerminal::Abandoned,
        1 => QueryColdReleaseTerminal::ReleaseOutstanding,
        _ => return Err(Denial::MalformedFrame),
    };
    Ok(QueryColdRelease {
        subject,
        fence,
        terminal,
    })
}
