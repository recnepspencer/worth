//! Actual typed creation, with real program-token and field initialization refusals.
pub use super::{handler::Handler, operation::*};
pub use std::{marker::PhantomData, sync::Mutex};

pub type Created<L> = WorthQueryApplicationCreatedRelation<GraphSchema<L>, Precedes, Node, Node>;
pub struct PayloadHandler<L: LawPolicy> {
    pub prior: Mutex<Option<Created<L>>>,
    pub marker: PhantomData<fn() -> L>,
}
impl<L: LawPolicy> Default for PayloadHandler<L> {
    fn default() -> Self {
        Self {
            prior: Mutex::new(None),
            marker: PhantomData,
        }
    }
}
impl<L: LawPolicy> OperationHandler<GraphSchema<L>, EditBinding<L>> for PayloadHandler<L> {
    fn decide(
        &self,
        input: &EdgeEdit,
        reader: &mut DecisionReader<'_, '_, '_, GraphSchema<L>, EditBinding<L>>,
    ) -> HandlerResult<Targets<L>, GraphDenial> {
        Handler.decide(input, reader)
    }
    fn build_candidate(
        &self,
        input: &EdgeEdit,
        targets: Targets<L>,
        writer: &mut CandidateWriter<'_, GraphSchema<L>, EditBinding<L>>,
    ) -> HandlerResult<Edited, GraphDenial> {
        let result = self.initialize(input, targets, writer);
        match result {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
impl<L: LawPolicy> PayloadHandler<L> {
    pub fn initialize(
        &self,
        input: &EdgeEdit,
        targets: Targets<L>,
        writer: &mut CandidateWriter<'_, GraphSchema<L>, EditBinding<L>>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        for (index, (from, to)) in targets.added.into_iter().enumerate() {
            let from = writer.projected_entity(&from)?;
            let to = writer.projected_entity(&to)?;
            let key = format!("payload-{}-{index}", input.serial);
            let target = writer.link_for_initialization(Precedes::reference(), &key, &from, &to)?;
            if input.serial == 4 {
                // Same-program duplicate key is a separate creation refusal.
                writer.link_for_initialization(Precedes::reference(), &key, &from, &to)?;
            }
            if input.serial == 2 {
                let prior = self.prior.lock().unwrap();
                writer.initialize_relation_field(
                    prior.as_ref().expect("prior successful program's token"),
                    EdgeWeight::reference(),
                    ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(2)
                        .unwrap(),
                )?;
            } else {
                writer.initialize_relation_field(
                    &target,
                    EdgeWeight::reference(),
                    ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(
                        input.serial,
                    )
                    .unwrap(),
                )?;
                if input.serial == 3 {
                    writer.initialize_relation_field(
                        &target,
                        EdgeWeight::reference(),
                        ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(3)
                            .unwrap(),
                    )?;
                }
            }
            *self.prior.lock().unwrap() = Some(target);
        }
        Ok(())
    }
}

#[path = "payload/court.rs"]
pub mod court;
