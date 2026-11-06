//! Moved whole from `domain_computation::primary_graph::application_attempt::commit_authority_binding`: the items below and every impl they own.
use crate::equality_type_id::EqualityTypeId as TypeId;
use std::any::Any;
use std::sync::Arc;
use worth_query_declaration::facade::application_operation::ApplicationSharedInput;

/// Exact capability input retained from the original operation admission.
///
/// The erased storage lets the non-generic commit receipt carry application
/// input and its funded allocation through receipt and recovery custody.
/// Correction preparation accesses the original binding's encoded carrier.
#[derive(Clone)]
pub struct WorthQueryRetainedGovernedInput {
    encoded: Option<Arc<dyn Any + Send + Sync>>,
    _value: Arc<dyn Any + Send + Sync>,
    type_id: TypeId,
    type_name: &'static str,
    governed_identity: Option<[u8; 32]>,
    original_scope: Option<Arc<crate::domain_computation::vocabulary::primary_graph::retained_selector::WorthQueryObservedScopeSelector>>,
    correction:
        Option<Arc<crate::domain_computation::vocabulary::application_aftermath::original::OriginalCorrectionDisclosure>>,
}

impl WorthQueryRetainedGovernedInput {
    #[expect(
        clippy::disallowed_methods,
        reason = "Type identity and names serve point lookup or diagnostics; canonical order uses declared identity"
    )]
    pub(in crate::domain_computation) fn from_admission<Input>(
        input: &ApplicationSharedInput<Input>,
        governed_identity: Option<[u8; 32]>,
        original_scope: Option<&Arc<crate::domain_computation::vocabulary::primary_graph::retained_selector::WorthQueryObservedScopeSelector>>,
        correction: Option<
            &Arc<crate::domain_computation::vocabulary::application_aftermath::original::OriginalCorrectionDisclosure>,
        >,
        encoded: Option<&Arc<dyn Any + Send + Sync>>,
    ) -> Self
    where
        Input: Send + Sync + 'static,
    {
        Self {
            encoded: encoded.cloned(),
            _value: Arc::new(input.clone()),
            type_id: TypeId::of::<Input>(),
            type_name: std::any::type_name::<Input>(),
            governed_identity,
            original_scope: original_scope.cloned(),
            correction: correction.cloned(),
        }
    }

    pub(in crate::domain_computation) fn downcast_ref<Input: 'static>(&self) -> Option<&Input> {
        (self.type_id == TypeId::of::<Input>())
            .then(|| {
                self._value
                    .downcast_ref::<ApplicationSharedInput<Input>>()
                    .map(AsRef::as_ref)
            })
            .flatten()
    }

    pub(in crate::domain_computation) fn encoded_input<Binding>(
        &self,
    ) -> Option<
        &worth_query_declaration::facade::application_operation::ApplicationEncodedInput<Binding>,
    >
    where
        Binding:
            worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding,
    {
        self.encoded.as_ref()?.downcast_ref()
    }

    pub(crate) const fn governed_identity(&self) -> Option<[u8; 32]> {
        self.governed_identity
    }

    pub(in crate::domain_computation) fn original_scope(
        &self,
    ) -> Option<&Arc<crate::domain_computation::vocabulary::primary_graph::retained_selector::WorthQueryObservedScopeSelector>>{
        self.original_scope.as_ref()
    }
    pub(in crate::domain_computation) fn correction(
        &self,
    ) -> Option<&crate::domain_computation::vocabulary::application_aftermath::original::OriginalCorrectionDisclosure>
{
        self.correction.as_deref()
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "Type identity and names serve point lookup or diagnostics; canonical order uses declared identity"
    )]
    pub(crate) fn axis_probe(governed_identity: [u8; 32]) -> Self {
        Self {
            encoded: None,
            _value: Arc::new(()),
            type_id: TypeId::of::<()>(),
            type_name: std::any::type_name::<()>(),
            governed_identity: Some(governed_identity),
            original_scope: None,
            correction: None,
        }
    }
}

impl std::fmt::Debug for WorthQueryRetainedGovernedInput {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryRetainedGovernedInput")
            .field("type_name", &self.type_name)
            .field("governed_identity", &self.governed_identity)
            .finish_non_exhaustive()
    }
}

impl PartialEq for WorthQueryRetainedGovernedInput {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id && self.governed_identity == other.governed_identity
    }
}

impl Eq for WorthQueryRetainedGovernedInput {}

impl WorthQueryRetainedGovernedInput {
    #[doc(hidden)]
    pub fn axis_probe_for_test(governed_identity: [u8; 32]) -> Self {
        Self::axis_probe(governed_identity)
    }
}

mod court_doors_for_test;
