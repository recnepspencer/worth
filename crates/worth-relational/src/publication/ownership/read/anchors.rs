//! The position owner is an independent current anchor, including after cold load.
use super::*;
use crate::history::data::current_route_presence::Posture;
use crate::history::data::{RequiredOutputPublicationPositions, RequiredOutputRoutePosition};
use std::collections::HashMap;
pub(in crate::publication::ownership) fn validate(
    roster: &mut CurrentPublicationRoster,
    positions: &RequiredOutputPublicationPositions,
    records: &[RecordRef],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    let mut live = HashMap::new();
    for (_, p) in &roster.positions {
        checkpoint(context, 1)?;
        let key = (
            &p.meaning.family,
            p.meaning.source_scope,
            p.meaning.source_partition,
        );
        if !p.valid_digest() || live.insert(key, p).is_some() {
            return Err(explain(
                corrupt(p.publication, RecordRef::Entity(p.meaning.source_scope)),
                "invalid or duplicate route digest",
            ));
        }
        let expected = match p.posture {
            Posture::Published => RequiredOutputRoutePosition::Published(p.publication),
            Posture::CodecRetired { retirement } => {
                RequiredOutputRoutePosition::CodecRetired(retirement)
            }
        };
        if positions
            .get(&p.meaning, context)?
            .is_some_and(|selected| selected != expected)
        {
            return Err(explain(
                corrupt(p.publication, RecordRef::Entity(p.meaning.source_scope)),
                "warm position differs from durable route",
            ));
        }
    }
    for record in records {
        if let RecordRef::Entity(source) = record {
            for expected in positions.current_owned.at(*source) {
                checkpoint(context, 1)?;
                if live
                    .get(&(
                        &expected.meaning.family,
                        *source,
                        expected.meaning.source_partition,
                    ))
                    .is_none_or(|p| p.publication != expected.publication)
                {
                    return Err(explain(
                        corrupt(expected.publication, RecordRef::Entity(*source)),
                        "warm route has no durable presence",
                    ));
                }
            }
        }
    }
    let mut postures = Vec::with_capacity(roster.publications.len());
    for row in &roster.publications {
        checkpoint(context, 1)?;
        let o = &row.value.origin;
        let p = live
            .remove(&(
                &o.meaning.family,
                o.meaning.source_scope,
                o.meaning.source_partition,
            ))
            .ok_or_else(|| {
                explain(
                    corrupt(
                        o.commit.commit_id,
                        RecordRef::Entity(o.meaning.source_scope),
                    ),
                    "publication lacks independent route presence",
                )
            })?;
        if p.meaning != o.meaning
            || p.publication != o.commit.commit_id
            || p.generation != o.commit.version_id
            || p.row_digest != row.digest
        {
            return Err(explain(
                corrupt(
                    o.commit.commit_id,
                    RecordRef::Entity(o.meaning.source_scope),
                ),
                "route generation or row digest differs",
            ));
        }
        postures.push(p.posture == Posture::Published);
    }
    if let Some(p) = live
        .into_values()
        .min_by_key(|p| (p.publication, p.meaning.source_scope))
    {
        return Err(corrupt(
            p.publication,
            RecordRef::Entity(p.meaning.source_scope),
        ));
    }
    for (row, posture) in std::mem::take(&mut roster.publications)
        .into_iter()
        .zip(postures)
    {
        if posture {
            roster.publications.push(row);
        } else {
            roster.codec_retired.push(row);
        }
    }
    Ok(())
}

fn explain(mut error: TransactionCommitError, detail: &str) -> TransactionCommitError {
    if let TransactionCommitError::Publication { error, .. } = &mut error {
        error.detail.push_str(": ");
        error.detail.push_str(detail);
    }
    error
}
