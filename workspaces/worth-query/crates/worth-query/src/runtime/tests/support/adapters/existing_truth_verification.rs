use super::*;
use std::collections::BTreeMap;

use worth_foundational::facade::{prepare_aspect_value_identity_basis, AspectValue};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct TestExistingTruthKey {
    binding_digest: String,
    target_collection: String,
    aspect_touch: WorthQueryAspectTouch,
}

impl TestExistingTruthKey {
    fn new(
        binding: &WorthQueryExistingTruthTargetBinding,
        aspect_touch: WorthQueryAspectTouch,
    ) -> Self {
        Self {
            binding_digest: binding.binding_digest(),
            target_collection: binding
                .terminal_target_collection_projection()
                .unwrap_or("none")
                .to_string(),
            aspect_touch,
        }
    }
}

#[derive(Default)]
pub(in crate::runtime::tests) struct TestExistingTruthVerificationAdapter {
    values: BTreeMap<TestExistingTruthKey, AspectValue>,
}

impl TestExistingTruthVerificationAdapter {
    pub(in crate::runtime::tests) fn with_value(
        mut self,
        binding: &WorthQueryExistingTruthTargetBinding,
        touch_fixture: &str,
        value: AspectValue,
    ) -> Self {
        let aspect_touch = test_aspect_touch(touch_fixture);
        self.values
            .insert(TestExistingTruthKey::new(binding, aspect_touch), value);
        self
    }

    fn lookup(
        &self,
        binding: &WorthQueryExistingTruthTargetBinding,
        aspect_touch: &WorthQueryAspectTouch,
    ) -> Option<&AspectValue> {
        self.values
            .get(&TestExistingTruthKey::new(binding, aspect_touch.clone()))
    }
}

impl WorthQueryRuntimeExistingTruthVerificationAdapter for TestExistingTruthVerificationAdapter {
    fn verify_existing_truth_assertion(
        &self,
        _request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &WorthQueryExistingTruthTargetBinding,
        aspects: &[WorthQueryAuthoredAspectMutation],
    ) -> Result<(), WorthQueryExistingTruthAssertionDenial> {
        for aspect in aspects {
            let aspect_touch = aspect.aspect_touch();
            if aspect.clears_existing_value() {
                return Err(WorthQueryExistingTruthAssertionDenial::new(
                    binding,
                    WorthQueryExistingTruthAssertionDenialKind::ClearAssertionUnsupported,
                    Some(aspect_touch.clone()),
                    None,
                    None,
                    "backend-verified assertions cannot clear authoritative truth",
                ));
            }
            let Some(found) = self.lookup(binding, &aspect_touch) else {
                return Err(WorthQueryExistingTruthAssertionDenial::new(
                    binding,
                    WorthQueryExistingTruthAssertionDenialKind::MissingAssertedAspect,
                    Some(aspect_touch.clone()),
                    Some(aspect.terminal_digest_material()),
                    None,
                    "authoritative truth did not contain the asserted aspect",
                ));
            };
            let Some(expected) = aspect.foundational_value() else {
                return Err(WorthQueryExistingTruthAssertionDenial::new(
                    binding,
                    WorthQueryExistingTruthAssertionDenialKind::MissingAssertedAspect,
                    Some(aspect_touch.clone()),
                    Some(aspect.terminal_digest_material()),
                    None,
                    "asserted aspect did not retain a native value",
                ));
            };
            if found != expected {
                return Err(WorthQueryExistingTruthAssertionDenial::new(
                    binding,
                    WorthQueryExistingTruthAssertionDenialKind::AssertedValueMismatch,
                    Some(aspect_touch.clone()),
                    Some(aspect.terminal_digest_material()),
                    Some(terminal_digest_from_aspect_value(found)),
                    "authoritative truth did not match the asserted value",
                ));
            }
        }
        Ok(())
    }

    fn probe_existing_truth(
        &self,
        request: &WorthQueryExistingTruthProbeRequest,
    ) -> Result<Vec<WorthQueryExistingTruthProbeField>, WorthQueryExistingTruthProbeDenial> {
        let mut fields = Vec::with_capacity(request.aspect_touches().len());
        for aspect_touch in request.aspect_touches() {
            let Some(value) = self.lookup(request.binding(), aspect_touch) else {
                return Err(WorthQueryExistingTruthProbeDenial::new(
                    request.binding(),
                    WorthQueryExistingTruthProbeDenialKind::MissingProbedAspect,
                    Some(aspect_touch.clone()),
                    "authoritative truth did not contain the probed aspect",
                ));
            };
            let field = WorthQueryExistingTruthProbeField::from_admitted_aspect_touch(
                aspect_touch.clone(),
                value.clone(),
            );
            fields.push(field);
        }
        Ok(fields)
    }
}

fn terminal_digest_from_aspect_value(value: &AspectValue) -> String {
    prepare_aspect_value_identity_basis(value)
        .as_str()
        .to_owned()
}

#[derive(Default)]
pub(in crate::runtime::tests) struct PermissiveExistingTruthVerificationAdapter;

impl WorthQueryRuntimeExistingTruthVerificationAdapter
    for PermissiveExistingTruthVerificationAdapter
{
    fn verify_existing_truth_assertion(
        &self,
        _request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        _binding: &WorthQueryExistingTruthTargetBinding,
        _aspects: &[WorthQueryAuthoredAspectMutation],
    ) -> Result<(), WorthQueryExistingTruthAssertionDenial> {
        Ok(())
    }

    fn probe_existing_truth(
        &self,
        request: &WorthQueryExistingTruthProbeRequest,
    ) -> Result<Vec<WorthQueryExistingTruthProbeField>, WorthQueryExistingTruthProbeDenial> {
        Ok(request
            .aspect_touches()
            .iter()
            .map(|aspect_touch| {
                WorthQueryExistingTruthProbeField::from_admitted_aspect_touch(
                    aspect_touch.clone(),
                    crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                        "permissive",
                    ),
                )
            })
            .collect())
    }
}
