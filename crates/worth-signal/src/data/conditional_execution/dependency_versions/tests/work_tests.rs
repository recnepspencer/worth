use super::*;
use crate::data::conditional_execution::dependency_versions::cache;
use crate::data::retained_storage::RetainedStoragePreparation as Work;

#[test]
fn dependency_cache_reads_and_records_use_the_existing_attempt_allowance() {
    for present in [false, true] {
        let mut graph = SignalGraph::new();
        let node = graph.create_node();
        let owner = SignalAspectLoweringOwner::fresh();
        graph.claim_aspect_lowering_owner(&owner).unwrap();
        let aspects = AspectMask::from(Aspect::new(1));
        let contract = install(&mut graph, &owner, node, aspects);
        if present {
            cache::record_dependency_versions(&mut graph, &contract, &mut Work::new(1_000_000))
                .unwrap();
        }
        // Native retention converts the source to the forked map substrate.
        // The sibling retained partition stays alive throughout the checks.
        let _partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
        {
            let selected = &mut graph;
            let source = selected.conditional_dependency_versions.clone();
            let mut measured_graph = selected.conditional_dependency_versions.fork_persistent();
            std::mem::swap(
                &mut measured_graph,
                &mut selected.conditional_dependency_versions,
            );
            let mut measured = Work::new(1_000_000);
            cache::record_dependency_versions(selected, &contract, &mut measured).unwrap();
            let cost = measured.visits();
            selected.conditional_dependency_versions = measured_graph;
            for available in [cost - 1, cost] {
                let original = selected.conditional_dependency_versions.fork_persistent();
                let before = selected.conditional_dependency_versions.clone();
                let mut work = Work::new(cost + 23);
                work.reserve_visits(cost + 23 - available).unwrap();
                let result = cache::record_dependency_versions(selected, &contract, &mut work);
                if available == cost {
                    result.unwrap();
                    assert_eq!(
                        cache::for_aspects(selected, node, aspects, &mut Work::new(1_000_000))
                            .unwrap(),
                        Some(AspectVersion::zero())
                    );
                    assert_eq!(
                        cache::for_aspects(
                            selected,
                            node,
                            AspectMask::ALL,
                            &mut Work::new(1_000_000)
                        )
                        .unwrap(),
                        None
                    );
                } else {
                    assert_eq!(
                        result,
                        Err(SignalError::ConditionalEvaluationWorkExhausted {
                            maximum_visits: cost + 23
                        })
                    );
                    assert_eq!(selected.conditional_dependency_versions, before);
                }
                selected.conditional_dependency_versions = original;
            }
            assert_eq!(selected.conditional_dependency_versions, source);
            let mut measured = Work::new(1_000_000);
            let expected = cache::for_aspects(selected, node, aspects, &mut measured).unwrap();
            assert_eq!(expected.is_some(), present);
            let cost = measured.visits();
            for available in [cost - 1, cost] {
                let mut work = Work::new(cost + 7);
                work.reserve_visits(cost + 7 - available).unwrap();
                let result = cache::for_aspects(selected, node, aspects, &mut work);
                if available == cost {
                    assert_eq!(result.unwrap(), expected);
                } else {
                    assert_eq!(
                        result,
                        Err(SignalError::ConditionalEvaluationWorkExhausted {
                            maximum_visits: cost + 7
                        })
                    );
                }
            }
        }
        assert_eq!(
            graph.conditional_dependency_versions.get(&node).is_some(),
            present
        );
    }
}
