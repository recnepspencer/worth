//! Canonical conditional continuation record bytes.
use super::super::{
    codec::{business_original, Reader},
    QueryManagedCleanupOriginal, QueryObligationDenial,
};
use super::*;
use worth_foundational::facade::HomeIdentity;

pub(in crate::query_obligations) fn encode(
    bytes: &mut Vec<u8>,
    record: &QueryConditionalDecisionRecord,
) {
    let (tag, key, intent) = match record {
        QueryConditionalDecisionRecord::Retain(row) => (0, &row.key, &row.intent),
        QueryConditionalDecisionRecord::Finish { key, intent } => (1, key, intent),
    };
    bytes.push(tag);
    bytes.extend_from_slice(key);
    bytes.extend_from_slice(intent);
    if let QueryConditionalDecisionRecord::Retain(row) = record {
        bytes.push(match row.posture {
            QueryConditionalPosture::CleanupRequired(_) => 0,
            QueryConditionalPosture::Retryable => 1,
            QueryConditionalPosture::CommitRetryable => 2,
            QueryConditionalPosture::ProductStale => 3,
            QueryConditionalPosture::Indeterminate(_) => 4,
        });
        match row.posture {
            QueryConditionalPosture::CleanupRequired(identity) => {
                bytes.extend_from_slice(&identity.acquire_position().get().to_le_bytes());
            }
            QueryConditionalPosture::Indeterminate(original) => {
                business_original::encode(bytes, recorded_original(original));
            }
            QueryConditionalPosture::Retryable
            | QueryConditionalPosture::CommitRetryable
            | QueryConditionalPosture::ProductStale => {}
        }
    }
}

pub(in crate::query_obligations) fn decode(
    reader: &mut Reader<'_>,
    home: HomeIdentity,
) -> Result<QueryConditionalDecisionRecord, QueryObligationDenial> {
    use QueryObligationDenial::MalformedFrame;
    let tag = reader.take::<1>()?[0];
    let key = reader.take()?;
    let intent = reader.take()?;
    Ok(match tag {
        0 => QueryConditionalDecisionRecord::Retain(QueryConditionalDecision {
            key,
            intent,
            posture: match reader.take::<1>()?[0] {
                0 => QueryConditionalPosture::CleanupRequired(
                    ManagedCleanupObligationIdentity::from_recorded(
                        home,
                        std::num::NonZeroU64::new(u64::from_le_bytes(reader.take()?))
                            .ok_or(MalformedFrame)?,
                    ),
                ),
                1 => QueryConditionalPosture::Retryable,
                2 => QueryConditionalPosture::CommitRetryable,
                3 => QueryConditionalPosture::ProductStale,
                4 => QueryConditionalPosture::Indeterminate(
                    match business_original::decode(reader)? {
                        None => QueryConditionalOriginal::Unclassified,
                        Some(QueryManagedCleanupOriginal::Attempt(attempt)) => {
                            QueryConditionalOriginal::Recovered(attempt)
                        }
                        Some(QueryManagedCleanupOriginal::Unpublished(_)) => {
                            return Err(MalformedFrame)
                        }
                    },
                ),
                _ => return Err(MalformedFrame),
            },
        }),
        1 => QueryConditionalDecisionRecord::Finish { key, intent },
        _ => return Err(MalformedFrame),
    })
}

/// The shared World-original representation: an attempt, or none yet.
fn recorded_original(original: QueryConditionalOriginal) -> Option<QueryManagedCleanupOriginal> {
    match original {
        QueryConditionalOriginal::Unclassified => None,
        QueryConditionalOriginal::Recovered(attempt) => {
            Some(QueryManagedCleanupOriginal::Attempt(attempt))
        }
    }
}
