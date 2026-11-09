use worth_query_host::facade::domain::{self, AspectValue, FieldKey, InternedString};

use super::super::contract::TemporalReadyNode;

pub struct Predicate;

impl domain::WorthQueryAuthoredHostConditionalPredicateProvider<TemporalReadyNode> for Predicate {
    fn authored_provider_description(&self) -> domain::BridgeAuthoredProviderDescription {
        domain::BridgeAuthoredProviderDescription::new(
            domain::BoundaryProtocolIdentity::new("worth.query.example.product.predicate"),
            domain::BoundaryProtocolVersion::new(1),
            Vec::<u8>::new(),
        )
    }
    fn readmit_authored_provider(
        &self,
        source: &domain::BridgeAuthoredProviderDescription,
    ) -> Result<Self, domain::AuthoredProviderReconstructionDenial> {
        self.authored_provider_description()
            .admit_source_version(source)?;
        if !source.payload().is_empty() {
            return Err(domain::AuthoredProviderReconstructionDenial::MalformedPayload);
        }
        Ok(Self)
    }
}

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

pub fn evaluate(
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
