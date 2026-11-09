//! Authored inverse Create and exact declared relation restoration.
use super::*;
impl<S: Scenario> OperationHandler<S, RestoreBinding<S>> for Handler {
    fn decide(
        &self,
        input: &Restore,
        reader: &mut DecisionReader<'_, '_, '_, S, RestoreBinding<S>>,
    ) -> HandlerResult<relation_splice::Restoration<S>, ()> {
        match relation_splice::read_restoration(input, reader) {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Restore,
        target: relation_splice::Restoration<S>,
        writer: &mut CandidateWriter<'_, S, RestoreBinding<S>>,
    ) -> HandlerResult<(), ()> {
        let result: Result<(), WorthQueryApplicationAttemptDenial> = (|| {
            let scope = writer.projected_entity(&target.scope)?;
            let created = writer.create_entity_in_context(
                &scope,
                schema::node::<S>(),
                WorthQueryApplicationEntityKey::new(if S::MODE == 12 {
                    "WORTH.coupled.0.0.19:body-cell-successor".to_owned()
                } else {
                    format!("authored-{}", input.id)
                })
                .unwrap(),
            )?;
            writer.initialize_field(&created, schema::id::<S>(), input.id)?;
            writer.initialize_field(&created, schema::value::<S>(), input.value)?;
            if S::MODE == 43 {
                writer
                    .initialize_content(
                        &created,
                        content::field::<S>().expect("the content scenario declares its field"),
                        &(input.id * 100),
                    )
                    .expect("valid installed word content");
            }
            writer.create_output::<Created<S>>(&created)?;
            <S::Heterogeneous as heterogeneous::Profile>::restore(input, &scope, writer)?;
            relation_splice::write_restoration(target.relations, &created, writer)?;
            for (index, &id) in input.co_anchors.iter().enumerate() {
                let co = writer.create_entity_in_context(
                    &scope,
                    schema::node::<S>(),
                    WorthQueryApplicationEntityKey::new(format!("authored-{id}")).unwrap(),
                )?;
                writer.initialize_field(&co, schema::id::<S>(), id)?;
                writer.initialize_field(&co, schema::value::<S>(), input.co_values[index])?;
                match index {
                    0 => writer.create_output::<CoCreated<S, 0>>(&co)?,
                    1 => writer.create_output::<CoCreated<S, 1>>(&co)?,
                    _ => unreachable!(),
                }
            }
            Ok(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
