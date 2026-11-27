//! A created timestamp initialized from the operation's trusted admission
//! instant: the Home's runtime time source read once per advancement, never
//! an input member, a literal or the host's own clock.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial,
    ApplicationOperationBodyCreatedField as Step, ApplicationOperationBodyCreationBuilder,
    ApplicationOperationBodyProgram,
};
use crate::application_schema::{
    fragment::RecordSelection, ApplicationFieldPresence, ApplicationScalarValueBinding,
    ApplicationSchemaDeclarationDenial, ApplicationSchemaMember as Member,
    CanonicalTimestampApplicationValueBinding as Timestamp,
};
use worth_foundational::facade::CanonicalTimestamp;

impl<Schema, Operation, Input, Entity>
    ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
{
    /// Initializes a required `CanonicalTimestamp` field of the created row
    /// with the instant this operation was admitted at, in the same commit.
    ///
    /// It is the instant a handler reads from `DecisionReader::admitted_at()`
    /// and a commit invariant from its context's `admitted_at()`: one reading
    /// of the Home's runtime time source per advancement, shared by every
    /// stage of an edit envelope. The caller supplies no time. An unavailable
    /// trusted time source refuses the operation; nothing is fabricated.
    /// ```
    /// use worth_query_declaration::facade::application_schema::{*, fragment::RecordSelection};
    /// fn stamp_birth<S, O, I, E, F>(
    ///     creation: ApplicationOperationBodyCreationBuilder<S, O, I, E>, created_at: F,
    /// ) -> Result<ApplicationOperationBodyCreationBuilder<S, O, I, E>, ApplicationOperationBodyAuthoringDenial>
    /// where F: RecordSelection<Entity = E, Value = CanonicalTimestamp,
    ///     Binding = CanonicalTimestampApplicationValueBinding>,
    /// { creation.initialize_from_admission_time(created_at) }
    /// ```
    /// A field of another value type does not compile:
    /// ```compile_fail
    /// use worth_query_declaration::facade::application_schema::{*, fragment::RecordSelection};
    /// fn wrong_value<S, O, I, E, F: RecordSelection<Entity = E, Value = u64>>(
    ///     creation: ApplicationOperationBodyCreationBuilder<S, O, I, E>, created_at: F,
    /// ) { let _ = creation.initialize_from_admission_time(created_at); }
    /// ```
    pub fn initialize_from_admission_time(
        mut self,
        field: impl RecordSelection<Entity = Entity, Value = CanonicalTimestamp, Binding = Timestamp>,
    ) -> Result<Self, Denial> {
        self.fields.push(step(field)?);
        Ok(self)
    }
}

impl<Schema, Operation, Input, Entity>
    super::TypedApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
{
    /// Typed-chain form of
    /// [`ApplicationOperationBodyCreationBuilder::initialize_from_admission_time`].
    pub fn initialize_from_admission_time(
        self,
        field: impl RecordSelection<Entity = Entity, Value = CanonicalTimestamp, Binding = Timestamp>,
    ) -> Self {
        self.step(|builder| builder.initialize_from_admission_time(field))
    }
}

/// The step for a root or child creation site: the target is required and
/// uses the native unitless timestamp codec, so the stamp is always present.
pub(super) fn step(
    field: impl RecordSelection<Value = CanonicalTimestamp, Binding = Timestamp>,
) -> Result<Step, Denial> {
    let checked = super::record_authoring::resolve(field)?;
    if checked.presence() != ApplicationFieldPresence::Required {
        return Err(Denial::IncompatibleSourceBinding);
    }
    Ok(Step::AdmissionTime {
        field: checked.locus().clone(),
        binding: Timestamp::IDENTITY,
    })
}

/// Cold closure for portable or authored bodies: an admission-time target is
/// a declared, required, unitless field of the native timestamp codec.
pub(super) fn validate(
    operation: &str,
    body: &ApplicationOperationBodyProgram,
    members: &[Member],
) -> Result<(), ApplicationSchemaDeclarationDenial> {
    let steps = body
        .creation()
        .into_iter()
        .flat_map(|root| root.initialization_sites())
        .flat_map(|site| site.fields())
        .filter(|step| step.is_admission_time());
    for step in steps {
        let target = step.field();
        let declared = members.iter().any(|member| matches!(member, Member::Field {
            entity, aspect, field, value_type, presence: ApplicationFieldPresence::Required, unit: None, ..
        } if entity == target.entity() && aspect == target.aspect() && field == target.field()
            && value_type == Timestamp::IDENTITY.as_str()));
        if !declared || *step.binding() != Timestamp::IDENTITY {
            return Err(ApplicationSchemaDeclarationDenial::program_body_dependency(
                operation,
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "admission_time_tests.rs"]
mod tests;
