use super::*;
use crate::facade::{DependencyEdge, SignalRuntimePolicy};
use crate::tests::support::{evaluate, version_ab};

struct EligiblePredicate(usize);
impl InstalledSignalConditionResolver for EligiblePredicate {
    fn resolve(
        &mut self,
        _: &crate::data::node::InstalledSignalConditionIdentity,
        _: &crate::logic::evaluation::ConditionEvaluationContext,
    ) -> Result<InstalledSignalConditionDecision, SignalError> {
        self.0 += 1;
        Ok(InstalledSignalConditionDecision::Eligible)
    }
}

#[test]
fn conditional_upstream_budget_denies_before_predicate_or_compute_with_exact_twin() {
    for maximum in [1, 2] {
        let (mut graph, contract) = installed_with(
            SignalConditionalCondition::RuntimePredicate,
            SignalConditionalArtifactReuse::NotReusable,
        );
        let sources = [graph.node().build(), graph.node().build()];
        for source in sources {
            evaluate(&mut graph, source, &mut |_, _| Ok(version_ab(0, 1))).unwrap();
        }
        graph
            .set_dependencies(
                contract.node(),
                sources.map(|source| DependencyEdge::new(source, Aspect::new(1))),
            )
            .unwrap();
        graph.set_runtime_policy(
            SignalRuntimePolicy::development().with_maximum_upstream_dependency_visits(maximum),
        );
        let mut partition = SignalEvaluationPartition::retain_basis_storage(&mut graph);
        let mut predicate = EligiblePredicate(0);
        let mut computes = 0;
        let (decision, observation, _rejected) = partition
            .execute_conditional(
                &mut graph,
                SignalConditionalExecutionRequest::new(&contract, "storage", "upstream", 1),
                &mut predicate,
                &mut DefaultComparatorPolicyResolver::default(),
                || {
                    computes += 1;
                    Ok(output(9))
                },
            )
            .unwrap()
            .into_parts();
        if maximum == 1 {
            let Err(failure) = decision else {
                panic!("short traversal must deny")
            };
            assert_eq!(
                failure.into_error(),
                SignalError::UpstreamDependencyWorkExhausted { maximum_visits: 1 }
            );
            assert_eq!((predicate.0, computes), (0, 0));
            assert!(observation.unwrap().is_none());
        } else {
            assert_eq!(
                decision.unwrap().class(),
                SignalConditionalDecisionClass::ComputedChanged
            );
            assert_eq!((predicate.0, computes), (1, 1));
            assert!(observation.unwrap().is_some());
        }
    }
}
