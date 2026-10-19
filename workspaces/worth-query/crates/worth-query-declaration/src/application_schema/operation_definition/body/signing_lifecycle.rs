//! One installed rule joins certified dependency edits to immutable consequence records.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyCreationBuilder,
};
use crate::application_schema::{
    ApplicationFieldBindingLocus as Locus, ApplicationFieldPresence, ApplicationFieldRef,
    ApplicationRelationRef, ApplicationSigningSupersession, ApplicationSigningSupersessionBinding,
    DeclaredApplicationFieldValue, NoApplicationUnit, OperationLinks, ReadOnly,
};
pub(super) mod validation;
pub(super) use validation::validate;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodySigningLifecycleSpec {
    pub history_relation: String,
    pub current_relation: String,
    pub source_entity: String,
    pub consequence_relation: String,
    pub consequence_field: Locus,
    pub maximum_evidence: usize,
}
impl ApplicationOperationBodySigningLifecycleSpec {
    pub fn canonical_record(&self) -> String {
        use std::fmt::Write;
        let mut record = String::from("body-signing-lifecycle-v1;");
        for text in [
            &self.history_relation,
            &self.current_relation,
            &self.source_entity,
            &self.consequence_relation,
            self.consequence_field.entity(),
            self.consequence_field.aspect(),
            self.consequence_field.field(),
        ] {
            write!(&mut record, "{}:{text};", text.len()).expect("String write");
        }
        write!(&mut record, "{};", self.maximum_evidence).expect("String write");
        record
    }
}
impl<Schema, Operation, Input, Entity>
    ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
{
    /// Bounds the deduplicated union of signable subjects affected by a source
    /// mutation, not a separately reset allowance for each declared capture path.
    pub fn bound_subject_impacts(
        mut self,
        maximum: std::num::NonZeroUsize,
    ) -> Result<Self, Denial> {
        if self.signing_lifecycle.is_none() || self.subject_impact_bound.is_some() {
            return Err(Denial::InvalidSigningLifecycle);
        }
        self.subject_impact_bound = Some(maximum);
        Ok(self)
    }
    /// The native owner, not each source mutator, publishes the declared consequences.
    pub fn supersede_on_subject_change<
        History,
        Current,
        Scope,
        Consequences,
        Record,
        Aspect,
        Field,
        Equality,
    >(
        self,
        history: ApplicationRelationRef<Schema, History, Scope, Entity>,
        current: ApplicationRelationRef<Schema, Current, Scope, Entity>,
        consequences: ApplicationRelationRef<Schema, Consequences, Entity, Record>,
        field: ApplicationFieldRef<
            Schema,
            Record,
            Aspect,
            Field,
            ApplicationSigningSupersession,
            ReadOnly,
            Equality,
            NoApplicationUnit,
        >,
        maximum_evidence: std::num::NonZeroUsize,
    ) -> Result<Self, Denial>
    where
        History: OperationLinks<Operation>,
        Current: OperationLinks<Operation>,
        Field: DeclaredApplicationFieldValue<
            Value = ApplicationSigningSupersession,
            Binding = ApplicationSigningSupersessionBinding,
        >,
    {
        if self.signing_lifecycle.is_some() || self.signing.is_none() {
            return Err(Denial::InvalidSigningLifecycle);
        }
        if Field::PRESENCE != ApplicationFieldPresence::Required {
            return Err(Denial::InvalidSigningLifecycle);
        }
        let spec = ApplicationOperationBodySigningLifecycleSpec {
            history_relation: history.name().to_owned(),
            current_relation: current.name().to_owned(),
            source_entity: history.from().to_owned(),
            consequence_relation: consequences.name().to_owned(),
            consequence_field: Locus::new(field.entity(), field.aspect(), field.field()),
            maximum_evidence: maximum_evidence.get(),
        };
        let mut creation = self.link_from_scope(history, "signing-history")?;
        creation = creation.link_from_scope(current, "signing-current")?;
        creation.signing_lifecycle = Some(spec);
        Ok(creation)
    }
}
