//! One actual incidence orbit across the generated member roster.
use super::*;
use std::collections::BTreeMap;
use worth_query_host::facade::domain::{OperationLinks, OperationReads, OperationUnlinks};

pub(super) fn unlink<B: ApplicationMutationBinding<Schema>>(
    writer: &mut CandidateWriter<'_, Schema, B>,
    prior: &BTreeMap<u64, observation::Member>,
) -> Result<(), WorthQueryApplicationAttemptDenial>
where
    OwnedNext: OperationReads<B::Operation> + OperationUnlinks<B::Operation>,
{
    for (from, to) in prior.values().zip(prior.values().cycle().skip(1)) {
        let from = writer.projected_entity(&from.row)?;
        let to = writer.projected_entity(&to.row)?;
        writer.unlink(OwnedNext::reference(), &from, &to)?;
    }
    Ok(())
}

pub(super) fn link<B: ApplicationMutationBinding<Schema>>(
    writer: &mut CandidateWriter<'_, Schema, B>,
    model: &str,
    members: &[WorthQueryApplicationEffectEntity<Schema, FirstOutput>],
) -> Result<(), WorthQueryApplicationAttemptDenial>
where
    OwnedNext: OperationLinks<B::Operation>,
{
    for (ordinal, (from, to)) in members
        .iter()
        .zip(members.iter().cycle().skip(1))
        .enumerate()
    {
        writer.link(
            OwnedNext::reference(),
            format!("owned-orbit:{model}:{ordinal}"),
            from,
            to,
        )?;
    }
    Ok(())
}
