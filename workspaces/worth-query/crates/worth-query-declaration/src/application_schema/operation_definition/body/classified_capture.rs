//! Explicit signed-subject versus audit-context captures; both reuse native typed source initialization.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyCaptureRole as Role,
    ApplicationOperationBodyCreatedField as Step, ApplicationOperationBodyCreationBuilder,
};
use crate::application_schema::{
    ApplicationFieldPresence, ApplicationOperationBodyCreation,
    ApplicationSigningProvenanceEncoding as Encoding,
};

impl<Schema, Operation, Input, Entity>
    ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
{
    pub fn capture_from_scope<Source, Field>(
        self,
        source: Source,
        field: Field,
        role: Role,
    ) -> Result<Self, Denial>
    where
        Source: crate::application_schema::fragment::RecordSelection,
        Field: crate::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Value = Source::Value,
            Binding = Source::Binding,
        >,
    {
        if super::record_authoring::resolve(source)?.presence()
            != ApplicationFieldPresence::Required
            || super::record_authoring::resolve(field)?.presence()
                != ApplicationFieldPresence::Required
        {
            return Err(Denial::IncompatibleSourceBinding);
        }
        Ok(self
            .initialize_from_scope(source, field)?
            .classify_last_capture(role))
    }
    pub fn capture_from_present_scope<Source, Field>(
        self,
        source: Source,
        target: Field,
        role: Role,
    ) -> Result<Self, Denial>
    where
        Source: crate::application_schema::fragment::RecordSelection,
        Field: crate::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Value = Source::Value,
            Binding = Source::Binding,
        >,
    {
        Ok(self
            .initialize_from_present_scope(source, target)?
            .classify_last_capture(role))
    }

    fn classify_last_capture(mut self, role: Role) -> Self {
        let step = match self
            .fields
            .pop()
            .expect("typed source initializer just appended")
        {
            Step::ScopeField {
                field,
                binding,
                scalar_family,
                source,
            } => Step::CapturedScopeField {
                field,
                binding,
                scalar_family,
                source,
                source_presence: ApplicationFieldPresence::Required,
                role,
            },
            Step::PresentScopeField {
                field,
                binding,
                scalar_family,
                source,
            } => Step::CapturedScopeField {
                field,
                binding,
                scalar_family,
                source,
                source_presence: ApplicationFieldPresence::Optional,
                role,
            },
            _ => unreachable!("private role attachment follows only a typed source initializer"),
        };
        self.fields.push(step);
        self
    }
}

pub(super) fn validate_roles(fields: &[Step], encoding: Option<Encoding>) -> Result<(), Denial> {
    // Graph-certified captures are not admitted through the older scope-only
    // provenance profile. The graph owner installs its distinct encoding.
    if fields
        .iter()
        .any(|step| matches!(step, Step::CapturedPathField { .. }))
        && encoding != Some(Encoding::CertifiedGraphSubjectV3)
    {
        return Err(Denial::UnclassifiedSignedCapture);
    }
    let classified = fields.iter().any(|step| {
        matches!(
            step,
            Step::CapturedScopeField { .. } | Step::CapturedPathField { .. }
        )
    });
    let unclassified = fields.iter().any(|step| {
        matches!(
            step,
            Step::ScopeField { .. } | Step::PresentScopeField { .. }
        )
    });
    let certified = matches!(
        encoding,
        Some(Encoding::CertifiedSubjectV2 | Encoding::CertifiedGraphSubjectV3)
    );
    if (certified && unclassified) || (!certified && classified) {
        return Err(Denial::UnclassifiedSignedCapture);
    }
    Ok(())
}
pub(super) fn validate_creation(creation: &ApplicationOperationBodyCreation) -> Result<(), Denial> {
    if creation.signing().is_some()
        && creation
            .fields()
            .iter()
            .any(|step| step.related_source().is_some() || matches!(step, Step::Computed { .. }))
    {
        return Err(Denial::UnclassifiedSignedCapture);
    }
    let encoding = creation.signing().and_then(|signing| {
        creation.fields().iter().find_map(|step| match step {
            Step::SigningProvenance { field, binding } if field == signing.provenance() => {
                Encoding::for_binding(binding)
            }
            _ => None,
        })
    });
    validate_roles(creation.fields(), encoding)
}
