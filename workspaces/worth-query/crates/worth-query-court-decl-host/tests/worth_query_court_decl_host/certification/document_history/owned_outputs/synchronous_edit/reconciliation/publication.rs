//! Reconcile member identities, survivor positions and temporary retirement facts.
use super::*;
use worth_query_host::facade::domain::{
    OperationCreates, OperationDeletes, OperationLinks, OperationReads, OperationUnlinks,
};
pub(super) fn publish<B: ApplicationMutationBinding<Schema>>(
    writer: &mut CandidateWriter<'_, Schema, B>,
    source: &WorthQueryApplicationEffectEntity<Schema, Model>,
    root: &WorthQueryApplicationEffectEntity<Schema, FirstOutput>,
    mut prior: observation::Prepared,
    count: u64,
) -> Result<(), WorthQueryApplicationAttemptDenial>
where
    FirstOf: OperationLinks<B::Operation>
        + OperationReads<B::Operation>
        + OperationUnlinks<B::Operation>,
    ChildOfFirst: OperationLinks<B::Operation>
        + OperationReads<B::Operation>
        + OperationUnlinks<B::Operation>,
    ReconciledSourceOf: OperationLinks<B::Operation>
        + OperationReads<B::Operation>
        + OperationUnlinks<B::Operation>,
    SecondOf: OperationLinks<B::Operation>
        + OperationReads<B::Operation>
        + OperationUnlinks<B::Operation>,
    FirstOutput: OperationCreates<B::Operation> + OperationDeletes<B::Operation>,
    ChildOutput: OperationCreates<B::Operation> + OperationDeletes<B::Operation>,
    SecondOutput: OperationCreates<B::Operation> + OperationDeletes<B::Operation>,
{
    let model = &prior.model;
    for retired in prior.retired {
        let row = writer.projected_entity(&retired.row)?;
        writer.unlink(ChildOfFirst::reference(), root, &row)?;
        writer.delete_entity(ChildOutput::reference(), &row)?;
        writer.retire_member::<Rows<B::Output, ChildOutput>>(&retired.key, &row)?;
    }
    for ordinal in 0..count {
        let key = observation::member_key(model, ordinal);
        let value = if ordinal == 0 {
            42
        } else {
            (ordinal + 1) * 1000 / (count + 1)
        };
        if let Some(prior) = prior.members.remove(&ordinal) {
            let row = writer.projected_entity(&prior.row)?;
            writer.write_field(&row, FirstValue::at("FirstValue"), value)?;
            writer.preserve_member::<Rows<B::Output, FirstOutput>>(&key, &row)?;
            let stable_source = writer.projected_entity(&prior.source)?;
            writer.preserve_member::<Rows<B::Output, SecondOutput>>(
                &prior.source_key,
                &stable_source,
            )?;
        } else {
            let row = writer.create_entity_in_context(
                source,
                FirstOutput::reference(),
                WorthQueryApplicationEntityKey::new(&key).unwrap(),
            )?;
            writer.initialize_field(&row, FirstKey::at("FirstKey"), key.clone())?;
            writer.initialize_field(&row, FirstValue::at("FirstValue"), value)?;
            writer.link(FirstOf::reference(), format!("of:{key}"), &row, source)?;
            writer.create_member::<Rows<B::Output, FirstOutput>>(&key, &row)?;
            let source_key = observation::source_key(model, ordinal);
            let stable_source = writer.create_entity_in_context(
                source,
                SecondOutput::reference(),
                WorthQueryApplicationEntityKey::new(&source_key).unwrap(),
            )?;
            writer.initialize_field(
                &stable_source,
                SecondKey::at("SecondKey"),
                source_key.clone(),
            )?;
            writer.initialize_field(&stable_source, SecondValue::at("SecondValue"), 0u64)?;
            writer.link(
                SecondOf::reference(),
                format!("of:{source_key}"),
                &stable_source,
                source,
            )?;
            writer.link(
                ReconciledSourceOf::reference(),
                format!("source:{key}"),
                &row,
                &stable_source,
            )?;
            writer.create_member::<Rows<B::Output, SecondOutput>>(&source_key, &stable_source)?;
        }
    }
    for (ordinal, prior) in prior.members {
        let row = writer.projected_entity(&prior.row)?;
        for child in &prior.children {
            let child_row = writer.projected_entity(&child.row)?;
            writer.unlink(ChildOfFirst::reference(), &row, &child_row)?;
            writer.delete_entity(ChildOutput::reference(), &child_row)?;
            writer.retire_member::<Rows<B::Output, ChildOutput>>(&child.key, &child_row)?;
        }
        writer.unlink(FirstOf::reference(), &row, source)?;
        let stable_source = writer.projected_entity(&prior.source)?;
        writer.unlink(ReconciledSourceOf::reference(), &row, &stable_source)?;
        writer.unlink(SecondOf::reference(), &stable_source, source)?;
        writer.delete_entity(SecondOutput::reference(), &stable_source)?;
        writer.retire_member::<Rows<B::Output, SecondOutput>>(&prior.source_key, &stable_source)?;
        writer.delete_entity(FirstOutput::reference(), &row)?;
        writer.retire_member::<Rows<B::Output, FirstOutput>>(&prior.key, &row)?;
        let key = format!("reconciled:retired:{model}:{ordinal}");
        let fact = writer.create_entity_in_context(
            source,
            ChildOutput::reference(),
            WorthQueryApplicationEntityKey::new(&key).unwrap(),
        )?;
        writer.initialize_field(&fact, ChildKey::at("ChildKey"), key.clone())?;
        writer.initialize_field(&fact, ChildValue::at("ChildValue"), ordinal)?;
        writer.link(ChildOfFirst::reference(), format!("of:{key}"), root, &fact)?;
        writer.create_member::<Rows<B::Output, ChildOutput>>(&key, &fact)?;
    }
    Ok(())
}
