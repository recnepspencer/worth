use worth_query_host::facade::domain::{self, AspectValue, FieldKey, InternedString};

use super::super::contract::TemporalReadyNode;

pub struct Predicate;
pub struct ReplacementPredicate;

impl domain::WorthQueryHostConditionalPredicateProvider<TemporalReadyNode> for Predicate {
    const SEMANTIC_IDENTITY: &'static str = "worth.query.example.product.predicate";

    fn retained_heap_bytes(
        &self,
    ) -> Result<
        domain::WorthQueryHostProviderHeapRetention,
        domain::WorthQueryHostProviderRetentionOverflow,
    > {
        Ok(domain::WorthQueryHostProviderHeapRetention::none())
    }

    fn evaluate(
        &self,
        observation: domain::WorthQueryConditionalObservationView<'_>,
    ) -> Result<domain::WorthQueryHostPredicateDecision, domain::WorthQueryHostPredicateFailure>
    {
        evaluate(observation)
    }
}

impl domain::WorthQueryHostConditionalPredicateProvider<TemporalReadyNode>
    for ReplacementPredicate
{
    const SEMANTIC_IDENTITY: &'static str = "worth.query.example.product.replacement-predicate";

    fn retained_heap_bytes(
        &self,
    ) -> Result<
        domain::WorthQueryHostProviderHeapRetention,
        domain::WorthQueryHostProviderRetentionOverflow,
    > {
        Ok(domain::WorthQueryHostProviderHeapRetention::none())
    }

    fn evaluate(
        &self,
        observation: domain::WorthQueryConditionalObservationView<'_>,
    ) -> Result<domain::WorthQueryHostPredicateDecision, domain::WorthQueryHostPredicateFailure>
    {
        evaluate(observation)
    }
}

fn evaluate(
    observation: domain::WorthQueryConditionalObservationView<'_>,
) -> Result<domain::WorthQueryHostPredicateDecision, domain::WorthQueryHostPredicateFailure> {
    let ready = observation.dependency(0).is_some_and(|dependency| {
        let domain::WorthQueryConditionalObservedValue::Present(artifact) = dependency.current()
        else {
            return false;
        };
        matches!(
            artifact.field(&FieldKey::new("IntentGateField").expect("valid field key")),
            Some(AspectValue::String(InternedString::Raw(value))) if value == "ready"
        ) && artifact
            .field(&FieldKey::new("IntentEffectField").expect("valid field key"))
            .is_none()
    });
    Ok(if ready {
        domain::WorthQueryHostPredicateDecision::Satisfied
    } else {
        domain::WorthQueryHostPredicateDecision::Unsatisfied
    })
}
