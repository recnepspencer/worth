use crate::domain_computation::primary_graph::application_checkpoint::facts::payload::sequence::FactSequence;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use crate::domain_computation::primary_graph::application_attempt::check_request_live;

pub fn bind_currentness_facts<Schema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    read_set: &mut super::super::WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        super::super::WorthQueryProjectedApplicationMutation,
    >,
    currentness_facts: &(impl FactSequence + ?Sized),
    subject: &str,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    check_request_live(read_set.admission.publication_request(), subject)?;
    ensure_current(execution, read_set, currentness_facts, subject)?;
    let mut merged = std::collections::BTreeMap::new();
    // Completion's original prefix owns the installed projection ordinals.
    // Index dependencies for conflict checks without rearranging that prefix.
    for (ordinal, fact) in read_set.facts.iter().enumerate() {
        let locator = fact.dependency_key();
        match merged.entry(locator) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert((false, ordinal));
            }
            std::collections::btree_map::Entry::Occupied(entry) => {
                let (_, existing) = *entry.get();
                if read_set.facts.get(existing) != Some(fact) {
                    return Err(currentness_mismatch(subject));
                }
            }
        }
    }
    let mut appended = Vec::new();
    for (ordinal, fact) in currentness_facts.iter().enumerate() {
        match merged.entry(fact.dependency_key()) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert((true, appended.len()));
                appended.push(ordinal);
            }
            std::collections::btree_map::Entry::Occupied(entry) => {
                let (additional, position) = *entry.get();
                let existing = if additional {
                    currentness_facts.get(appended[position])
                } else {
                    read_set.facts.get(position)
                };
                if existing != Some(fact) {
                    return Err(currentness_mismatch(subject));
                }
            }
        }
    }
    // The completed-read owner retains original backing and replaces facts
    // with their matching effect routes only after admission succeeds.
    read_set
        .admission
        .validate_current_authority()
        .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
    read_set.append_completed_facts(
        appended.into_iter().map(|ordinal| {
            currentness_facts
                .get(ordinal)
                .expect("indexed currentness fact")
                .clone()
        }),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
}

pub fn ensure_current<Schema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    read_set: &super::super::WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        super::super::WorthQueryProjectedApplicationMutation,
    >,
    currentness_facts: &(impl FactSequence + ?Sized),
    subject: &str,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let current = read_set.lease.handle().with_runtime(execution, |runtime| {
        for fact in currentness_facts.iter() {
            check_request_live(read_set.admission.publication_request(), subject)?;
            if !fact.remains_equal_in(runtime, execution, read_set.lease.snapshot())? {
                return Ok::<bool, WorthQueryApplicationAttemptDenial>(false);
            }
        }
        Ok(true)
    })?;
    if current {
        Ok(())
    } else {
        Err(WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch,
            subject,
        ))
    }
}

fn currentness_mismatch(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch,
        subject,
    )
}
