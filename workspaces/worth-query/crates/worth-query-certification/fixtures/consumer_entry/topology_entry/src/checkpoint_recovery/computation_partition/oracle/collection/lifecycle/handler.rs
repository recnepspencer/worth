//! Both lifecycle handlers borrow the same installed checked computation.
use super::binding::{CreatedSupport, CreatedTotal, PreservedSupport, PreservedTotal};
use super::*;
use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial, PositiveLength};
use worth_query_host::facade::primary_graph::WorthQueryInvariantEntityIdentity;

type Run<Schema, Binding> = Box<
    dyn Fn(
            &mut DecisionReader<'_, '_, '_, Schema, Binding>,
            &WorthQueryInvariantEntityIdentity<Schema, facts::EntrySet>,
        ) -> WorthQueryDecisionPlan<Option<u64>, PlanarMutationDenial>
        + Send
        + Sync,
>;

macro_rules! handler {
    ($handler:ident, $binding:ident, $publish:ident $(, $observe:ident)?) => {
        pub(super) struct $handler<Schema: TopologySchemaBinding>(
            Option<Run<Schema, $binding<Schema>>>,
        );
        impl<Schema: TopologySchemaBinding> $handler<Schema> {
            pub(super) fn idle() -> Self {
                Self(None)
            }
            pub(super) fn running(
                run: impl Fn(
                        &mut DecisionReader<'_, '_, '_, Schema, $binding<Schema>>,
                        &WorthQueryInvariantEntityIdentity<Schema, facts::EntrySet>,
                    ) -> WorthQueryDecisionPlan<Option<u64>, PlanarMutationDenial>
                    + Send
                    + Sync
                    + 'static,
            ) -> Self {
                Self(Some(Box::new(run)))
            }
        }
        impl<Schema: TopologySchemaBinding> OperationHandler<Schema, $binding<Schema>>
            for $handler<Schema>
        {
            fn decide(
                &self,
                _: &RegionOutputInput,
                _: &mut DecisionReader<'_, '_, '_, Schema, $binding<Schema>>,
            ) -> HandlerResult<u64, PlanarMutationDenial> {
                HandlerResult::DomainDenied(PlanarMutationDenial::MissingCoordinate)
            }
            fn prepare_decision(
                &self,
                input: &RegionOutputInput,
                reader: &mut DecisionReader<'_, '_, '_, Schema, $binding<Schema>>,
            ) -> WorthQueryDecisionPlan<u64, PlanarMutationDenial> {
                let Some(run) = &self.0 else {
                    return WorthQueryDecisionPlan::immediate(HandlerResult::DomainDenied(
                        PlanarMutationDenial::MissingCoordinate,
                    ));
                };
                $(if let Err(error) = $observe(input, reader) {
                    return WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(error));
                })?
                let set = match reader
                    .resolve_entity(facts::EntrySetKey::reference(), input.entries.clone())
                {
                    Ok(set) => set,
                    Err(error) => {
                        return WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(
                            error,
                        ))
                    }
                };
                run(reader, &set).and_then(|value| match value {
                    Some(bits) => HandlerResult::Completed(bits),
                    None => HandlerResult::DomainDenied(PlanarMutationDenial::MissingCoordinate),
                })
            }
            fn candidate_requirements(
                &self,
                _: &RegionOutputInput,
                _: &u64,
            ) -> ApplicationCandidateRequirements {
                $binding::<Schema>::CANDIDATES.fixed_requirements().expect("fixed candidate contract")
            }
            fn build_candidate(
                &self,
                input: &RegionOutputInput,
                bits: u64,
                writer: &mut CandidateWriter<'_, Schema, $binding<Schema>>,
            ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
                let value = f64::from_bits(bits);
                // This fixture stores integer totals in the existing PositiveLength field.
                if !value.is_finite()
                    || value.fract() != 0.0
                    || !(1.0..=(1_u64 << 53) as f64).contains(&value)
                {
                    return HandlerResult::DomainDenied(PlanarMutationDenial::MissingCoordinate);
                }
                let value =
                    PositiveLength::new(value as u64).expect("checked positive integer total");
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
handler!(InitialHandler, InitialBinding, create);
handler!(
    PreserveHandler,
    PreserveBinding,
    preserve,
    observe_preserved_topology
);

fn observe_preserved_topology<Schema: TopologySchemaBinding>(
    input: &RegionOutputInput,
    reader: &mut DecisionReader<'_, '_, '_, Schema, PreserveBinding<Schema>>,
) -> Result<(), HandlerExecutionDenial> {
    let key = output_key(&input.scope_key);
    for member_key in [key.clone(), format!("{key}:b"), format!("{key}:c")] {
        // Selection alone grants nothing: the real decision reader resolves
        // and records each native target for the subsequent candidate writer.
        let body = reader.resolve_entity(BodyKey::reference(), member_key)?;
        // Replacement lowering needs the actual field preimage, in addition
        // to the observed entity. The total still comes only from the collection.
        reader.field(&body, Length::reference())?;
    }
    Ok(())
}

fn create<Schema: TopologySchemaBinding>(
    writer: &mut CandidateWriter<'_, Schema, InitialBinding<Schema>>,
    key: &str,
    value: PositiveLength,
) -> Result<(), HandlerExecutionDenial> {
    let mut bodies = Vec::new();
    for (suffix, x, y) in [("", 1, 1), ("b", 2, 1), ("c", 1, 2)] {
        let member_key = if suffix.is_empty() {
            key.to_owned()
        } else {
            format!("{key}:{suffix}")
        };
        let body = writer
            .create_entity(
                Body::reference(),
                primary_graph::WorthQueryApplicationEntityKey::new(member_key.clone())
                    .map_err(HandlerExecutionDenial::new)?,
            )
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&body, BodyKey::reference(), member_key)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&body, PositionX::reference(), length(x))
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&body, PositionY::reference(), length(y))
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&body, Length::reference(), value)
            .map_err(HandlerExecutionDenial::new)?;
        if suffix.is_empty() {
            writer
                .create_output::<CreatedTotal<Schema>>(&body)
                .map_err(HandlerExecutionDenial::new)?;
        } else {
            writer
                .create_member::<CreatedSupport<Schema>>(suffix, &body)
                .map_err(HandlerExecutionDenial::new)?;
        }
        bodies.push(body);
    }
    for index in 0..3 {
        writer
            .link(
                PlanarSuccessor::reference(),
                format!("{key}:successor:{index}"),
                &bodies[index],
                &bodies[(index + 1) % 3],
            )
            .map_err(HandlerExecutionDenial::new)?;
    }
    Ok(())
}

fn preserve<Schema: TopologySchemaBinding>(
    writer: &mut CandidateWriter<'_, Schema, PreserveBinding<Schema>>,
    key: &str,
    value: PositiveLength,
) -> Result<(), HandlerExecutionDenial> {
    for suffix in ["", "b", "c"] {
        let member_key = if suffix.is_empty() {
            key.to_owned()
        } else {
            format!("{key}:{suffix}")
        };
        let body = writer
            .resolve_entity(BodyKey::reference(), member_key)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .write_field(&body, Length::reference(), value)
            .map_err(HandlerExecutionDenial::new)?;
        if suffix.is_empty() {
            writer
                .preserve_output::<PreservedTotal<Schema>>(&body)
                .map_err(HandlerExecutionDenial::new)?;
        } else {
            writer
                .preserve_member::<PreservedSupport<Schema>>(suffix, &body)
                .map_err(HandlerExecutionDenial::new)?;
        }
    }
    Ok(())
}
