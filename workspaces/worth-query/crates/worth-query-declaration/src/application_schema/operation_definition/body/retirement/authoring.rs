//! Typed permissions and endpoint roles precede authority-free retirement meaning.
use super::ApplicationOperationBodyOwnedSelectionRetirement as Retirement;
use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyBuilder,
    ApplicationRelationRef, ApplicationScalarValueBinding, DeclaredApplicationFieldValue,
    OperationDeletes, OperationReads, OperationUnlinks,
};
use std::num::NonZeroUsize;
impl<S, O, I> ApplicationOperationBodyBuilder<S, O, I> {
    /// Retires owned current selections only when this body's exact source input changes.
    /// Holders and scope survive. The completed body must write the same field/member.
    /// Signed creation is not supported by this unsigned source-consequence profile.
    pub fn retire_owned_selections_on_input_change<
        Root,
        A,
        F,
        V,
        W,
        Q,
        U,
        Owners,
        Owner,
        Selection,
        Selected,
    >(
        mut self,
        trigger: ApplicationFieldRef<S, Root, A, F, V, W, Q, U>,
        member: &str,
        owners: ApplicationRelationRef<S, Owners, Root, Owner>,
        selection: ApplicationRelationRef<S, Selection, Owner, Selected>,
        maximum_owners: NonZeroUsize,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<O>,
        Owner: OperationReads<O>,
        Selected: OperationReads<O> + OperationDeletes<O>,
        F: OperationReads<O> + DeclaredApplicationFieldValue<Value = V>,
        U: ApplicationFieldUnit,
        Owners: OperationReads<O>,
        Selection: OperationReads<O> + OperationUnlinks<O>,
    {
        if member.is_empty()
            || member.trim() != member
            || owners.to() != selection.from()
            || owners.integrity().cardinality.target_max != Some(1)
            || selection.integrity().cardinality.source_max != Some(1)
            || selection.integrity().cardinality.target_max != Some(1)
        {
            return Err(Denial::InvalidSelectionRetirement);
        }
        self.retirements.push(Retirement {
            trigger: ApplicationFieldBindingLocus::new(
                trigger.entity(),
                trigger.aspect(),
                trigger.field(),
            ),
            binding: F::Binding::IDENTITY,
            scalar_family: F::Binding::SCALAR_FAMILY,
            member: member.into(),
            owners_relation: owners.name().into(),
            scope: owners.from().into(),
            owner: owners.to().into(),
            selection_relation: selection.name().into(),
            selected: selection.to().into(),
            maximum_owners,
        });
        Ok(self)
    }
}
