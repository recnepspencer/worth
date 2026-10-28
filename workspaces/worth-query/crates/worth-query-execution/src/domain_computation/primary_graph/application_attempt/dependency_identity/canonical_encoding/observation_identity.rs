//! One observation encoding, with signing's admitted canonical preparation and typed stops.
use super::{
    append_fact, push, text, unsigned, CanonicalBasisEntry, CanonicalBasisEntryKind,
    CanonicalBasisValue, CanonicalDigestId, WorthQueryApplicationObservedFact as Fact,
    WorthQueryCanonicalWorkEvidence, DOMAIN, INDEX_GUARD_RULE_VERSION, RULE_VERSION,
};
use crate::domain_computation::primary_graph::application_attempt::signed_creation::{
    canonical, identity, WorthQuerySignedCreationPreparationDenial as Denial,
};
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;

pub(in crate::domain_computation::primary_graph::application_attempt) fn authentication_observation_identity(
    operation: [u8; 32],
    facts: &(impl FactSequence + ?Sized),
    request: &worth_execution::ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<([u8; 32], WorthQueryCanonicalWorkEvidence), Denial> {
    // Assembly storage is already reserved by the enclosing signing owner.
    // Admit the bounded representation work before building the first entry,
    // and retain interruption points while traversing the original facts.
    let (rule, entries) = basis(operation, facts, || identity::check_interruption(context))?;
    canonical::derive(entries, DOMAIN, rule, request, context)
}

pub(super) fn basis<E>(
    operation: [u8; 32],
    facts: &(impl FactSequence + ?Sized),
    mut before_fact: impl FnMut() -> Result<(), E>,
) -> Result<(&'static str, Vec<CanonicalBasisEntry>), E> {
    let mut entries = Vec::new();
    push(
        &mut entries,
        "declared-key".into(),
        CanonicalBasisEntryKind::Identity,
        CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(operation)),
    );
    push(
        &mut entries,
        "fact-count".into(),
        CanonicalBasisEntryKind::Shape,
        unsigned(u64::try_from(facts.len()).unwrap_or(u64::MAX)),
    );
    for (index, fact) in facts.iter().enumerate() {
        before_fact()?;
        let prefix = format!("fact.{index}");
        push(
            &mut entries,
            format!("{prefix}.locator"),
            CanonicalBasisEntryKind::Locator,
            text(fact.locator_identity()),
        );
        if let Fact::Aggregate { observation } = fact {
            super::aggregate::append(&mut entries, &prefix, observation, &mut before_fact)?;
        } else {
            append_fact(&mut entries, &prefix, fact);
        }
    }
    let rule = if facts
        .iter()
        .any(|fact| matches!(fact, Fact::IndexedEntitySelection { .. }))
    {
        INDEX_GUARD_RULE_VERSION
    } else {
        RULE_VERSION
    };
    Ok((rule, entries))
}

#[cfg(test)]
mod tests {
    use super::{authentication_observation_identity, Denial};
    use crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution;
    use worth_query_admission::facade::authenticated_principal::{
        WorthQueryCancellationSource, WorthQueryRequestScope,
    };
    #[test]
    fn cancellation_after_kernel_entry_remains_a_typed_signing_execution_stop() {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(None, cancellation.token());
        let policy = worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, None),
        );
        let execution = QueryRequestExecution::open(
            worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(policy),
            &scope,
        );
        let request = execution.execution_request().unwrap();
        let entered = std::cell::Cell::new(false);
        let _stopped = execution.decision_stage(|context| {
            entered.set(true);
            cancellation.cancel();
            let denied = authentication_observation_identity([1; 32], &[], &request, context);
            assert!(matches!(denied, Err(Denial::Execution(_))));
        });
        assert!(
            entered.get(),
            "the cancellation occurs inside, not before, the native kernel"
        );
    }
}
