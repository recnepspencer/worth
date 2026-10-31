//! Both original decision work and candidate construction use ordinary public receivers.
use super::{mutation, query, schema::*};
use worth_query_host::facade::{application_entry::*, primary_graph::*};
#[derive(Clone, Copy)]
pub(super) enum DecisionRead {
    Declared,
    Empty,
    TwoProbes,
    OtherScope,
    Undeclared,
}
pub(super) struct Handler(pub DecisionRead);
impl OperationHandler<Catalog, mutation::Binding> for Handler {
    fn decide(
        &self,
        input: &mutation::Rename,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, mutation::Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Catalog, Item>, mutation::Denial> {
        let rows = match self.0 {
            DecisionRead::Declared | DecisionRead::Empty | DecisionRead::TwoProbes => {
                // compiled-example: begin
                let rows = reader.index_query::<query::PrefixBinding>(query::FindPrefix {
                    directory: input.directory.clone(),
                    prefix: input.prefix.clone(),
                });
                // compiled-example: end
                rows
            }
            DecisionRead::OtherScope => {
                reader.index_query::<query::PrefixBinding>(query::FindPrefix {
                    directory: "B".into(),
                    prefix: input.prefix.clone(),
                })
            }
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
