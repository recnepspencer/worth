//! Sealed test doors over this module's private members.
use super::*;

impl WorkflowAdmittedActor {
    #[doc(hidden)]
    pub fn append_fields_for_test(
        self,
        locators: &WorkflowActorFields,
        fields: &mut BTreeMap<AspectFieldLocator, AspectValue>,
    ) {
        Self::append_fields(self, locators, fields)
    }
}

impl WorkflowAdmittedActor {
    #[doc(hidden)]
    pub fn entity_for_test(self) -> EntityId {
        Self::entity(self)
    }
}

impl WorkflowAdmittedActor {
    #[doc(hidden)]
    pub fn from_admission_for_test<S, O, I, C>(
        admitted: &WorthQueryAdmittedApplicationOperation<S, O, I, C>,
    ) -> Self {
        Self::from_admission::<S, O, I, C>(admitted)
    }
}
