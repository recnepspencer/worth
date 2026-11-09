//! Typed permissions and endpoint roles precede authority-free retirement meaning.
use super::ApplicationOperationBodyOwnedSelectionRetirement as Retirement;
use crate::application_schema::{
    ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyBuilder,
    ApplicationRelationRef, ApplicationScalarValueBinding, OperationDeletes, OperationReads,
    OperationUnlinks,
};
use std::num::NonZeroUsize;
impl<S, O, I> ApplicationOperationBodyBuilder<S, O, I> {
    /// Retires owned current selections only when this body's exact source input changes.
    /// Holders and scope survive. The completed body must write the same field/member.
    /// Signed creation is not supported by this unsigned source-consequence profile.
    pub fn retire_owned_selections_on_input_change<Root, F, Owners, Owner, Selection, Selected>(
        mut self,
        trigger: F,
        member: &str,
        owners: ApplicationRelationRef<S, Owners, Root, Owner>,
        selection: ApplicationRelationRef<S, Selection, Owner, Selected>,
        maximum_owners: NonZeroUsize,
    ) -> Result<Self, Denial>
    where
        Root: OperationReads<O>,
        Owner: OperationReads<O>,
        Selected: OperationReads<O> + OperationDeletes<O>,
        Owners: OperationReads<O>,
        Selection: OperationReads<O> + OperationUnlinks<O>,
        F: crate::application_schema::fragment::RecordSelection<Entity = Root>,
    {
        let trigger = super::super::record_authoring::resolve(trigger)?;
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
            trigger: trigger.locus().clone(),
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
