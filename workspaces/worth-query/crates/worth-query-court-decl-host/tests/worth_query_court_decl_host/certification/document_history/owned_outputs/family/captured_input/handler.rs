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
/// A member of this authored weight makes the reader read `first:carriage:d`
/// without declaring it in its input capture (#254 fixture).
pub const UNDECLARED: u64 = 9;

thread_local! {
    static CAPTURES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}
/// Capture runs on this thread: a capture admits the producer's operation.
pub fn captures_on_this_thread() -> u64 {
    CAPTURES.with(std::cell::Cell::get)
}
pub fn count_capture() {
    CAPTURES.with(|count| count.set(count.get() + 1));
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
                handler::count_capture();
                let result = (|| {
                    let source = reader.scope().clone();
                    let members = reader.relations_from(ModelMember::reference(), &source)?;
                    let mut trigger_ready = true;
                    // A zero-weight control parent changes the authored choice
                    // only after its actual output exists on this candidate.
                    for member in &members {
                        if reader.field(member.to(), MemberValue::at("MemberValue"))? == Some(0) {
                            for link in
                                reader.relations_from(UpstreamOf::reference(), member.to())?
                            {
                                let key = reader
                                    .field(link.to(), ModelKey::at("ModelKey"))?
                                    .expect("required control key");
                                trigger_ready &= reader
                                    .resolve_optional_entity(
                                        FirstKey::at("FirstKey"),
                                        format!("first:{key}"),
                                    )?
                                    .is_some();
                            }
                        }
                    }
                    let mut parents = Vec::new();
                    for member in members {
                        let weight = reader
                            .field(member.to(), MemberValue::at("MemberValue"))?
                            .expect("required authored weight");
                        if weight == handler::UNDECLARED {
                            // Read, never declared, and unlinked: no discovery
                            // reaches this reader from that parent either.
                            parents.push(handler::ParentData {
                                key: "carriage:d".into(),
                                weight,
                            });
                            continue;
                        }
                        let links = reader.relations_from(UpstreamOf::reference(), member.to())?;
                        // Before the control is ready choose the largest
                        // authored priority; afterwards choose the smallest.
                        let mut selected = links.first();
                        if links.len() > 1 {
                            let mut priority = None;
                            for link in &links {
                                let value = reader
                                    .field(link.to(), ModelValue::at("ModelValue"))?
                                    .expect("required authored priority");
                                if priority.is_none_or(|old| {
                                    if trigger_ready {
                                        value < old
                                    } else {
                                        value > old
                                    }
                                }) {
                                    priority = Some(value);
                                    selected = Some(link);
                                }
                            }
                        }
                        if let Some(link) = selected {
                            let member = reader.entity_member(link.to())?;
                            let source =
                                member.as_entity().expect("this parent is an actual Entity");
                            let key = reader
                                .field(source, ModelKey::at("ModelKey"))?
                                .expect("required parent key");
                            let demand = first::Demand(key.clone());
                            // The control uses the first declared port, which
                            // this sequential wave drains before anchor rows.
                            if weight == 0 || weight % 2 == 1 {
                                reader.parent::<program::SeedConnection, _>(source, demand)?;
                            } else {
                                reader.parent::<program::AnchorConnection, _>(source, demand)?;
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
                        let key = format!("first:{}", parent.key);
                        let row = if parent.weight == handler::UNDECLARED {
                            match reader.resolve_optional_entity(FirstKey::at("FirstKey"), key)? {
                                Some(row) => row,
                                None => continue,
                            }
                        } else {
                            reader.resolve_entity(FirstKey::at("FirstKey"), key)?
                        };
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
