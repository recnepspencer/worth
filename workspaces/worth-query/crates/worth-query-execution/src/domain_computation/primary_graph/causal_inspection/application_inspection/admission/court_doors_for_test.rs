//! Sealed test doors over this module's private members.
use super::*;

impl<'a, 'storage, Schema, Scope> InspectionRead<'a, 'storage, Schema, Scope> {
    #[doc(hidden)]
    pub fn memory_for_test(&self) -> &&'storage mut Option<ExecutionMemoryReservation> {
        &self.memory
    }
}

impl<'a, 'storage, Schema, Scope> InspectionRead<'a, 'storage, Schema, Scope> {
    #[doc(hidden)]
    pub fn principal_for_test(&self) -> &worth_relational::facade::identity::EntityId {
        &self.principal
    }
}

impl<'a, 'storage, Schema, Scope> InspectionRead<'a, 'storage, Schema, Scope> {
    #[doc(hidden)]
    pub fn scope_for_test(&self) -> &&'a WorthQueryApplicationEntityIdentity<Schema, Scope> {
        &self.scope
    }
}

impl<'a, 'storage, Schema, Scope> InspectionRead<'a, 'storage, Schema, Scope> {
    #[doc(hidden)]
    pub fn session_for_test(&self) -> &WorthQueryManagedGraphWorkSession {
        &self.session
    }
}

impl<'a, 'storage, Schema, Scope> InspectionRead<'a, 'storage, Schema, Scope> {
    #[doc(hidden)]
    pub fn target_for_test(&self) -> &Arc<WorthQueryApplicationInspectionTarget> {
        &self.target
    }
}

impl<'a, 'storage, Schema, Scope> InspectionRead<'a, 'storage, Schema, Scope> {
    #[doc(hidden)]
    pub fn memory_mut_for_test(&mut self) -> &mut &'storage mut Option<ExecutionMemoryReservation> {
        &mut self.memory
    }
}

impl<'a, 'storage, Schema, Scope> InspectionRead<'a, 'storage, Schema, Scope> {
    #[doc(hidden)]
    /// The target and session beside the mutable memory slot: one borrow, three fields.
    pub fn target_session_memory_mut_for_test(
        &mut self,
    ) -> (
        &Arc<WorthQueryApplicationInspectionTarget>,
        &WorthQueryManagedGraphWorkSession,
        &mut &'storage mut Option<ExecutionMemoryReservation>,
    ) {
        (&self.target, &self.session, &mut self.memory)
    }
}
