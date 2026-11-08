//! One decision supplies both retirement and advancement of an owned route.
use super::*;
use crate::history::data::CanonicalRequiredOutputOrigin;
use crate::identity::data::EntityId;
use crate::mvcc::RequiredOutputRolePosture;
use std::sync::Arc;

type Successor<'a> = (usize, &'a Arc<CanonicalRequiredOutputOrigin>);

pub(super) fn select<'a>(
    candidate: &'a CanonicalCommitEnvelope,
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
                return Err(crate::mvcc::origin_denial(
                    "a live owned successor cannot publish into a deleted source scope",
                ));
            }
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
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
        let refused = crate::mvcc::origin_denial("posture read stopped");
        assert!(eligible(false, true, &mut [Err(refused)].into_iter()).is_err());
    }
}
