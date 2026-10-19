//! Candidate publication consumes the original checked owner's decision.
use super::*;
use members::{Quantity, QuantityKey, QuantitySnapshot};
type Run<B> = Box<
    dyn Fn(
            &mut DecisionReader<'_, '_, '_, Schema, B>,
            &WorthQueryInvariantEntityIdentity<Schema, Body>,
        ) -> WorthQueryDecisionPlan<Snapshot, PlanarMutationDenial>
        + Send
        + Sync,
>;
macro_rules! handler {
    ($handler:ident,$binding:ident,$publish:ident,$observe:expr) => {
        pub(super) struct $handler(pub(super) Run<binding::$binding>);
        impl OperationHandler<Schema, binding::$binding> for $handler {
            fn decide(
                &self,
                _: &binding::Input,
                _: &mut DecisionReader<'_, '_, '_, Schema, binding::$binding>,
            ) -> HandlerResult<Snapshot, PlanarMutationDenial> {
                HandlerResult::DomainDenied(PlanarMutationDenial::MissingCoordinate)
            }
            fn prepare_decision(
                &self,
                input: &binding::Input,
                reader: &mut DecisionReader<'_, '_, '_, Schema, binding::$binding>,
            ) -> WorthQueryDecisionPlan<Snapshot, PlanarMutationDenial> {
                if $observe {
                    let observed = (|| -> Result<(), HandlerExecutionDenial> {
                        let output = reader.resolve_entity(
                            QuantityKey::reference(),
                            output_key(&input.scope_key),
                        )?;
                        reader.field(&output, QuantitySnapshot::reference())?;
                        Ok(())
                    })();
                    if let Err(error) = observed {
                        return WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(
                            error,
                        ));
                    }
                }
                match reader.resolve_entity(BodyKey::reference(), input.scope_key.clone()) {
                    Ok(root) => (self.0)(reader, &root),
                    Err(error) => {
                        WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(error))
                    }
                }
            }
            fn candidate_requirements(
                &self,
                _: &binding::Input,
                _: &Snapshot,
            ) -> ApplicationCandidateRequirements {
                binding::$binding::CANDIDATES
                    .fixed_requirements()
                    .expect("fixed candidate contract")
            }
            fn build_candidate(
                &self,
                input: &binding::Input,
                value: Snapshot,
                writer: &mut CandidateWriter<'_, Schema, binding::$binding>,
            ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
                match $publish(writer, &output_key(&input.scope_key), value) {
                    Ok(()) => HandlerResult::Completed(PlanarAdjustmentResult {
                        changed_vertices: 1,
                    }),
                    Err(error) => HandlerResult::ExecutionDenied(error),
                }
            }
        }
    };
}
handler!(InitialHandler, Initial, create, false);
handler!(PreserveHandler, Preserve, preserve, true);
pub(super) fn output_key(scope: &str) -> String {
    format!("quantity:{scope}")
}
fn create(
    writer: &mut CandidateWriter<'_, Schema, binding::Initial>,
    key: &str,
    value: Snapshot,
) -> Result<(), HandlerExecutionDenial> {
    let target = writer
        .create_entity(
            Quantity::reference(),
            WorthQueryApplicationEntityKey::new(key).map_err(HandlerExecutionDenial::new)?,
        )
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(&target, QuantityKey::reference(), key.to_owned())
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(&target, QuantitySnapshot::reference(), value)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .create_output::<binding::Created>(&target)
        .map_err(HandlerExecutionDenial::new)
}
fn preserve(
    writer: &mut CandidateWriter<'_, Schema, binding::Preserve>,
    key: &str,
    value: Snapshot,
) -> Result<(), HandlerExecutionDenial> {
    let target = writer
        .resolve_entity(QuantityKey::reference(), key.to_owned())
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .write_field(&target, QuantitySnapshot::reference(), value)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .preserve_output::<binding::Preserved>(&target)
        .map_err(HandlerExecutionDenial::new)
}
