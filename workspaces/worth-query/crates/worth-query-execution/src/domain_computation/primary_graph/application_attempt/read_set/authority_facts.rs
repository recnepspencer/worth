//! Merge native owner observations without losing or contradicting handler facts.
use super::super::check_request_live;
use super::{
    denial, WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact, WorthQueryCompleteApplicationReadSet,
    WorthQueryProjectedApplicationMutation,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
{
    pub(in crate::domain_computation::primary_graph::application_attempt) fn append_authority_facts(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        incoming: &[WorthQueryApplicationObservedFact],
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.lease.handle().with_runtime(execution, |runtime| {
            for fact in incoming {
                check_request_live(
                    self.admission.publication_request(),
                    "workflow operation authority",
                )?;
                if !fact.remains_equal_in(runtime, execution, self.lease.snapshot())? {
                    return Err(denial(
                        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                        format!(
                            "workflow authority observation changed: {}",
                            fact.locator_identity()
                        ),
                    ));
                }
            }
            Ok(())
        })?;
        let mut locators = std::collections::BTreeMap::new();
        for (index, fact) in self.facts.iter().enumerate() {
            check_request_live(
                self.admission.publication_request(),
                "workflow operation authority",
            )?;
            let locator = fact.dependency_key();
            if locators.insert(locator, index).is_some() {
                return Err(denial(
                    WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                    "workflow candidate contains duplicate fact locators",
                ));
            }
        }
        let original_count = self.facts.len();
        let mut additions = Vec::new();
        for fact in incoming {
            check_request_live(
                self.admission.publication_request(),
                "workflow operation authority",
            )?;
            let locator = fact.dependency_key();
            match locators.entry(locator) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(original_count + additions.len());
                    additions.push(fact);
                }
                std::collections::btree_map::Entry::Occupied(entry)
                    if (if *entry.get() < original_count {
                        &self.facts[*entry.get()]
                    } else {
                        additions[*entry.get() - original_count]
                    }) != fact =>
                {
                    return Err(denial(
                        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                        "workflow authority fact conflicts with decision",
                    ));
                }
                std::collections::btree_map::Entry::Occupied(_) => {}
            }
        }
        check_request_live(
            self.admission.publication_request(),
            "workflow operation authority",
        )?;
        // References only: cloning nested source values retains its previous
        // semantic owner and is outside the inline array payload charge.
        self.append_completed_facts(additions.into_iter().cloned(), allocation_policy)?;
        Ok(())
    }
}
