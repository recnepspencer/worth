//! Both original decision work and candidate construction use ordinary public receivers.
use super::{mutation, probe, query, schema::*};
use worth_query_host::facade::primary_graph::*;
#[derive(Clone, Copy)]
pub(super) enum DecisionRead {
    Declared,
    Empty,
    TwoProbes,
    OtherScope,
    Undeclared,
    /// The handler's partitioned computation probes the Parent instead.
    Partitioned(probe::Probe),
    /// The declared shared-owner peer read of the Prefix Parent.
    Peer(&'static str),
    /// The Exact Parent, declared only for the operation's own scope, at a peer.
    UndeclaredPeer(&'static str),
}
pub(super) struct Handler(pub DecisionRead, pub probe::Installed);
impl OperationHandler<Catalog, mutation::Binding> for Handler {
    fn prepare_decision(
        &self,
        input: &mutation::Rename,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, mutation::Binding>,
    ) -> WorthQueryDecisionPlan<WorthQueryInvariantMutationTarget<Catalog, Item>, mutation::Denial>
    {
        let DecisionRead::Partitioned(probe) = self.0 else {
            return WorthQueryDecisionPlan::immediate(self.decide(input, reader));
        };
        let request = probe::ProbeRequest {
            directory: input.directory.clone(),
            prefix: input.prefix.clone(),
            probe,
        };
        let prepared = match self.1.prepare(reader, &request) {
            Ok(prepared) => prepared,
            Err(cause) => {
                if let worth_query_host::facade::application_contribution::WorthQueryPartitionedComputationDenial::Read(
                    worth_query_host::facade::application_contribution::WorthQueryComputationReadDenial::ManagedIndex(denial),
                ) = &cause
                {
                    probe::REFUSED.with(|refused| refused.borrow_mut().push(denial.clone()));
                }
                return WorthQueryDecisionPlan::immediate(HandlerResult::ExecutionDenied(
                    HandlerExecutionDenial::new(cause),
                ));
            }
        };
        let decided = mutation::Handler.decide(input, reader);
        let item = input.item.clone();
        prepared.into_decision_plan(move |completed| match (completed, decided) {
            (Ok(completed), HandlerResult::Completed(target)) => match completed.complete() {
                Ok(rows) => {
                    assert!(
                        rows.iter().any(|(identity, _)| *identity == item),
                        "the computation's committed Parent read actually selected the item"
                    );
                    HandlerResult::Completed(target)
                }
                Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
            },
            (Err(cause), _) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
            (_, other) => other,
        })
    }
    fn decide(
        &self,
        input: &mutation::Rename,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, mutation::Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Catalog, Item>, mutation::Denial> {
        let peer = match self.0 {
            DecisionRead::Peer(peer) => Some(reader.index_query::<query::PrefixBinding>(
                query::FindPrefix {
                    directory: peer.into(),
                    prefix: String::new(),
                },
            )),
            DecisionRead::UndeclaredPeer(peer) => Some(
                reader.index_query::<super::refinement::ExactBinding>(
                    super::refinement::FindExact {
                        directory: peer.into(),
                        name: String::new(),
                    },
                ),
            ),
            _ => None,
        };
        if let Some(rows) = peer {
            return match super::peer_scope::observe(rows) {
                Ok(_) => mutation::Handler.decide(input, reader),
                Err(cause) => HandlerResult::ExecutionDenied(cause),
            };
        }
        let rows = match self.0 {
            DecisionRead::Declared | DecisionRead::TwoProbes => {
                // compiled-example: begin
                let rows = reader.index_query::<query::PrefixBinding>(query::FindPrefix {
                    directory: input.directory.clone(),
                    prefix: input.prefix.clone(),
                });
                // compiled-example: end
                rows
            }
            DecisionRead::Empty => {
                // An independent genuine empty probe, even when the mutation's
                // explicit Source expectation names another admitted parameter.
                reader.index_query::<query::PrefixBinding>(query::FindPrefix {
                    directory: input.directory.clone(),
                    prefix: "needle".into(),
                })
            }
            DecisionRead::OtherScope => {
                reader.index_query::<query::PrefixBinding>(query::FindPrefix {
                    directory: "B".into(),
                    prefix: input.prefix.clone(),
                })
            }
            DecisionRead::Partitioned(_) => unreachable!("prepared through the computation"),
            DecisionRead::Peer(_) | DecisionRead::UndeclaredPeer(_) => unreachable!("read above"),
            DecisionRead::Undeclared => {
                reader.index_query::<query::SourceBinding>(query::ReadSource {
                    directory: input.directory.clone(),
                })
            }
        };
        match rows {
            Ok(rows) => {
                if matches!(self.0, DecisionRead::Empty) {
                    assert!(
                        rows.is_empty(),
                        "the original committed reader qualified an empty Parent"
                    );
                } else {
                    assert!(
                        rows.iter().any(|row| row.identity == input.item),
                        "the original committed reader actually selected x"
                    );
                }
                if matches!(self.0, DecisionRead::TwoProbes) {
                    let second = reader.index_query::<query::PrefixBinding>(query::FindPrefix {
                        directory: input.directory.clone(),
                        prefix: "sha".into(),
                    });
                    match second {
                        Ok(rows) => assert!(rows.iter().any(|row| row.identity == input.item)),
                        Err(cause) => return HandlerResult::ExecutionDenied(cause),
                    }
                }
                mutation::Handler.decide(input, reader)
            }
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &mutation::Rename,
        target: WorthQueryInvariantMutationTarget<Catalog, Item>,
        writer: &mut CandidateWriter<'_, Catalog, mutation::Binding>,
    ) -> HandlerResult<mutation::Renamed, mutation::Denial> {
        mutation::Handler.build_candidate(input, target, writer)
    }
}
