//! Inspect owner-issued observations without constructing or changing retained evidence.
use super::super::super::{cold_owned_revision::RequiredProgram, *};
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact as Fact;

pub(super) fn aggregate_sizes<S: Scenario>(
    host: &WorthQueryProgramApplicationRuntime<S, RequiredProgram>,
) -> Vec<usize> {
    let graph = host
        .runtime()
        .runtime_for_test()
        .primary_provider_for_test()
        .graph_for_test();
    let lineage = graph.output_lineage.lock().unwrap();
    let mut sizes = Vec::new();
    for branches in lineage.by_source_for_test().values() {
        for generations in branches.values() {
            for records in generations.values() {
                for record in records {
                    let Some(record) = record.get() else { continue };
                    let Some(facts) = record.observed_source_facts_for_test() else {
                        continue;
                    };
                    for fact in facts.postconditions_for_test().iter_for_test() {
                        if let Fact::Aggregate { observation } = fact {
                            sizes.push(observation.native().len());
                        }
                    }
                }
            }
        }
    }
    sizes
}
