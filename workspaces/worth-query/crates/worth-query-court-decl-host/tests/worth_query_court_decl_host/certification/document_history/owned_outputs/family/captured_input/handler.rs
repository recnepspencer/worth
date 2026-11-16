//! The original lifecycle handler consumes exactly its pure prelude's data.
use super::*;
#[derive(worth_foundational::facade::ChargedBytes)]
#[charged_bytes(crate = "worth_foundational::facade")]
pub struct ParentData {
    pub key: String,
    pub weight: u64,
}
#[derive(worth_foundational::facade::ChargedBytes)]
#[charged_bytes(crate = "worth_foundational::facade")]
pub struct Data {
    pub parents: Vec<ParentData>,
}

impl worth_execution::ChargedBytes for first::Demand {
    fn additional_charged_bytes(&self) -> u64 {
        self.0.capacity() as u64
    }
}

macro_rules! producer_capture_handler {
    () => {
        impl WorthQueryProducerInputCaptureHandler<Schema, Producer, Capture> for Handler {
            type Captured = handler::Data;
            fn capture_inputs(
                &self,
                _: &Input,
                reader: &mut WorthQueryProducerInputReader<'_, '_, '_, Schema, Binding, Capture>,
            ) -> HandlerResult<
                WorthQueryCapturedProducerInputs<Schema, Capture, handler::Data>,
                Denial,
            > {
                let result = (|| {
                    let source = reader.scope().clone();
                    let mut parents = Vec::new();
                    for member in reader.relations_from(ModelMember::reference(), &source)? {
                        let weight = reader
                            .field(member.to(), MemberValue::at("MemberValue"))?
                            .expect("required authored weight");
                        for link in reader.relations_from(UpstreamOf::reference(), member.to())? {
                            let key = reader
                                .field(link.to(), ModelKey::at("ModelKey"))?
                                .expect("required parent key");
                            let demand = first::Demand(key.clone());
                            if weight % 2 == 1 {
                                reader.parent::<program::SeedConnection, _>(link.to(), demand)?;
                            } else {
                                reader.parent::<program::AnchorConnection, _>(link.to(), demand)?;
                            }
                            parents.push(handler::ParentData { key, weight });
                        }
                    }
                    reader.finish(handler::Data { parents })
                })();
                match result {
                    Ok(data) => HandlerResult::Completed(data),
                    Err(cause) => HandlerResult::ExecutionDenied(cause),
                }
            }
            fn prepare_captured_decision(
                &self,
                input: &Input,
                captured: WorthQueryCapturedProducerInputs<Schema, Capture, handler::Data>,
                reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
            ) -> WorthQueryDecisionPlan<Decision, Denial> {
                let result = (|| {
                    let mut total = 0;
                    for parent in captured.into_data().parents {
                        let row = reader.resolve_entity(
                            FirstKey::at("FirstKey"),
                            format!("first:{}", parent.key),
                        )?;
                        let value = reader
                            .field(&row, FirstValue::at("FirstValue"))?
                            .expect("required ready parent value");
                        total += value * parent.weight;
                    }
                    Ok::<_, HandlerExecutionDenial>(total)
                })();
                WorthQueryDecisionPlan::immediate(match result {
                    Ok(total) => match self.decide(input, reader) {
                        HandlerResult::Completed(mut decision) => {
                            decision.member_value = total;
                            HandlerResult::Completed(decision)
                        }
                        other => other,
                    },
                    Err(cause) => HandlerResult::ExecutionDenied(cause),
                })
            }
        }
    };
}
