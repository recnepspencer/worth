//! One decision supplies both retirement and advancement of an owned route.
use worth_relational_contracts::internal_port as contracts;
use super::*;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin;
use contracts::identity::data::record_ids::EntityId;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture;
use std::sync::Arc;

type Successor<'a> = (usize, &'a Arc<CanonicalRequiredOutputOrigin>);

pub(super) fn select<'a>(
    candidate: &'a CanonicalCommitEnvelopeBody,
    scope: EntityId,
    scope_deleted: bool,
    writer: &mut Writer<'_, '_, '_, '_, '_>,
) -> Result<Vec<Successor<'a>>, TransactionCommitError> {
    let count = candidate.output_origins().count();
    writer.checkpoint(count.saturating_mul(2))?;
    writer.retain((count as u64).saturating_mul(std::mem::size_of::<Successor<'_>>() as u64))?;
    let mut selected = Vec::with_capacity(count);
    for (ordinal, origin) in candidate.output_origins().enumerate() {
        if origin.meaning.source_scope != scope {
            continue;
        }
        let mut members = origin
            .members
            .iter()
            .map(|member| writer.checkpoint(1).map(|()| member.posture));
        if eligible(scope_deleted, origin.source_scope_owned, &mut members)? {
            selected.push((ordinal, origin));
        }
    }
    Ok(selected)
}

fn eligible(
    scope_deleted: bool,
    source_scope_owned: bool,
    members: &mut dyn Iterator<Item = Result<RequiredOutputRolePosture, TransactionCommitError>>,
) -> Result<bool, TransactionCommitError> {
    if !source_scope_owned {
        return Ok(false);
    }
    for member in members {
        if member? != RequiredOutputRolePosture::Retire {
            if scope_deleted {
                return Err(
                    contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                        "a live owned successor cannot publish into a deleted source scope",
                    ),
                );
            }
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use RequiredOutputRolePosture::{Preserve, Retire};

    #[test]
    fn retirement_only_cannot_keep_custody_but_a_live_successor_can() {
        // Plain policy inputs, not an issued origin or canonical authority.
        for (deleted, owned, postures, expected) in [
            (false, true, vec![Retire, Retire], Some(false)),
            (false, true, vec![], Some(false)),
            (false, true, vec![Retire, Preserve], Some(true)),
            (false, false, vec![Preserve], Some(false)),
            (true, true, vec![Retire], Some(false)),
            (true, true, vec![Preserve], None),
        ] {
            let result = eligible(deleted, owned, &mut postures.into_iter().map(Ok));
            match expected {
                Some(expected) => assert_eq!(result.unwrap(), expected),
                None => assert!(result.is_err()),
            }
        }
        let refused =
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "posture read stopped",
            );
        assert!(eligible(false, true, &mut [Err(refused)].into_iter()).is_err());
    }
}
