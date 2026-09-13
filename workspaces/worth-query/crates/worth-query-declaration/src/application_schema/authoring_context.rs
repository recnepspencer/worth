use worth_foundational::facade::ScalarAspectType;

use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::{
    ApplicationOperationProgramTarget, ApplicationSchemaBindingIdentity, ApplicationSchemaMember,
    ApplicationSchemaMemberProvenance, ErasedApplicationSchemaDeclaration,
};

/// Exact installed schema meaning available to declaration authoring.
///
/// This context validates descriptors but grants no execution authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSchemaAuthoringContext {
    binding: ApplicationSchemaBindingIdentity,
    members: Vec<ApplicationSchemaMember>,
    member_provenance: ApplicationSchemaMemberProvenance,
}

impl ApplicationSchemaAuthoringContext {
    #[doc(hidden)]
    pub fn from_installed_declaration(
        binding: ApplicationSchemaBindingIdentity,
        declaration: &ErasedApplicationSchemaDeclaration,
        member_provenance: &ApplicationSchemaMemberProvenance,
    ) -> Self {
        Self {
            binding,
            members: declaration.members().to_vec(),
            member_provenance: member_provenance.clone(),
        }
    }

    pub fn binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding
    }

    pub(crate) fn admit_entity(
        &self,
        entity: &str,
    ) -> Result<(), ApplicationSchemaAuthoringDenial> {
        if self.members.iter().any(
            |member| matches!(member, ApplicationSchemaMember::Entity { entity: installed } if installed == entity),
        ) {
            Ok(())
        } else {
            Err(ApplicationSchemaAuthoringDenial::new(
                ApplicationSchemaAuthoringDenialKind::UnknownEntity,
                entity,
            ))
        }
    }

    pub(crate) fn admit_operation<Operation: 'static, Input: 'static>(
        &self,
        operation: &str,
        input_type: WorthQueryPortableTypeIdentity,
    ) -> Result<(), ApplicationSchemaAuthoringDenial> {
        if !self
            .member_provenance
            .admits_operation::<Operation, Input>(operation, &input_type)
        {
            return Err(ApplicationSchemaAuthoringDenial::new(
                ApplicationSchemaAuthoringDenialKind::OperationProvenanceMismatch,
                operation,
            ));
        }
        let installed = self.members.iter().find_map(|member| match member {
            ApplicationSchemaMember::Operation {
                operation: installed,
                input_type,
            } if installed == operation => Some(input_type),
            _ => None,
        });
        let Some(installed_input_type) = installed else {
            return Err(ApplicationSchemaAuthoringDenial::new(
                ApplicationSchemaAuthoringDenialKind::UnknownOperation,
                operation,
            ));
        };
        if installed_input_type != &input_type {
            return Err(ApplicationSchemaAuthoringDenial::new(
                ApplicationSchemaAuthoringDenialKind::OperationInputTypeMismatch,
                operation,
            ));
        }
        Ok(())
    }

    pub(crate) fn admit_effect<Effect: 'static, Payload: 'static>(
        &self,
        effect: &str,
        payload_type: WorthQueryPortableTypeIdentity,
    ) -> Result<(), ApplicationSchemaAuthoringDenial> {
        if !self
            .member_provenance
            .admits_effect::<Effect, Payload>(effect, &payload_type)
        {
            return Err(ApplicationSchemaAuthoringDenial::new(
                ApplicationSchemaAuthoringDenialKind::EffectProvenanceMismatch,
                effect,
            ));
        }
        let installed = self.members.iter().find_map(|member| match member {
            ApplicationSchemaMember::Effect {
                effect: installed,
                payload_type,
            } if installed == effect => Some(payload_type),
            _ => None,
        });
        let Some(installed_payload_type) = installed else {
            return Err(ApplicationSchemaAuthoringDenial::new(
                ApplicationSchemaAuthoringDenialKind::UnknownEffect,
                effect,
            ));
        };
        if installed_payload_type != &payload_type {
            return Err(ApplicationSchemaAuthoringDenial::new(
                ApplicationSchemaAuthoringDenialKind::EffectPayloadTypeMismatch,
                effect,
            ));
        }
        Ok(())
    }

    pub(crate) fn admit_relation(
        &self,
        relation: &str,
        from: &str,
        to: &str,
    ) -> Result<(), ApplicationSchemaAuthoringDenial> {
        if self.members.iter().any(|member| {
            matches!(
                member,
                ApplicationSchemaMember::Relation {
                    relation: installed,
                    from: installed_from,
                    to: installed_to,
                    ..
                } if installed == relation && installed_from == from && installed_to == to
            )
        }) {
            Ok(())
        } else {
            Err(ApplicationSchemaAuthoringDenial::new(
                ApplicationSchemaAuthoringDenialKind::UnknownRelation,
                relation,
            ))
        }
    }

    pub(crate) fn admit_operation_program(
        &self,
        operation: &str,
        admission: ApplicationOperationProgramAdmission<'_>,
    ) -> Result<(), ApplicationSchemaAuthoringDenial> {
        if self.members.iter().any(|member| {
            matches!(
                member,
                ApplicationSchemaMember::OperationProgram {
                    operation: installed,
                    target,
                } if installed == operation && admission.matches(target)
            )
        }) {
            Ok(())
        } else {
            Err(ApplicationSchemaAuthoringDenial::new(
                ApplicationSchemaAuthoringDenialKind::OperationProgramNotInstalled,
                operation,
            ))
        }
    }

    pub(crate) fn admit_field(
        &self,
        field: ApplicationFieldAdmission<'_>,
    ) -> Result<(), ApplicationSchemaAuthoringDenial> {
        let installed = self.members.iter().find(|member| {
            matches!(
                member,
                ApplicationSchemaMember::Field {
                    entity,
                    aspect,
                    field: installed_field,
                    ..
                } if entity == field.entity
                    && aspect == field.aspect
                    && installed_field == field.field
            )
        });
        let Some(installed) = installed else {
            return Err(ApplicationSchemaAuthoringDenial::new(
                ApplicationSchemaAuthoringDenialKind::UnknownField,
                field.field,
            ));
        };
        validate_field_capability(installed, &field)
    }
}

fn validate_field_capability(
    installed: &ApplicationSchemaMember,
    field: &ApplicationFieldAdmission<'_>,
) -> Result<(), ApplicationSchemaAuthoringDenial> {
    let ApplicationSchemaMember::Field {
        scalar_family,
        value_type,
        unit,
        writable,
        equality_queryable,
        ..
    } = installed
    else {
        unreachable!("field admission requires a field member")
    };
    if *scalar_family != field.scalar_family {
        return field_denial(
            ApplicationSchemaAuthoringDenialKind::FieldFamilyMismatch,
            field,
        );
    }
    if value_type != field.value_type {
        return field_denial(
            ApplicationSchemaAuthoringDenialKind::FieldValueTypeMismatch,
            field,
        );
    }
    if unit.as_deref() != field.unit {
        return field_denial(
            ApplicationSchemaAuthoringDenialKind::FieldUnitMismatch,
            field,
        );
    }
    if field.requires_write && !writable {
        return field_denial(
            ApplicationSchemaAuthoringDenialKind::FieldNotWritable,
            field,
        );
    }
    if field.requires_equality && !equality_queryable {
        return field_denial(
            ApplicationSchemaAuthoringDenialKind::FieldNotEqualityQueryable,
            field,
        );
    }
    Ok(())
}

fn field_denial(
    kind: ApplicationSchemaAuthoringDenialKind,
    field: &ApplicationFieldAdmission<'_>,
) -> Result<(), ApplicationSchemaAuthoringDenial> {
    Err(ApplicationSchemaAuthoringDenial::new(kind, field.field))
}

pub(crate) struct ApplicationFieldAdmission<'a> {
    pub entity: &'a str,
    pub aspect: &'a str,
    pub field: &'a str,
    pub scalar_family: ScalarAspectType,
    pub value_type: &'a str,
    pub unit: Option<&'a str>,
    pub requires_write: bool,
    pub requires_equality: bool,
}

pub(crate) enum ApplicationOperationProgramAdmission<'a> {
    Create(&'a str),
    Delete(&'a str),
    Write {
        entity: &'a str,
        aspect: &'a str,
        field: &'a str,
    },
    Link {
        relation: &'a str,
        from: &'a str,
        to: &'a str,
    },
    Unlink {
        relation: &'a str,
        from: &'a str,
        to: &'a str,
    },
    Emit(&'a str),
}

impl ApplicationOperationProgramAdmission<'_> {
    fn matches(&self, target: &ApplicationOperationProgramTarget) -> bool {
        match (self, target) {
            (Self::Create(expected), ApplicationOperationProgramTarget::Create { entity })
            | (Self::Delete(expected), ApplicationOperationProgramTarget::Delete { entity }) => {
                *expected == entity
            }
            (
                Self::Write {
                    entity,
                    aspect,
                    field,
                },
                ApplicationOperationProgramTarget::Write {
                    entity: installed_entity,
                    aspect: installed_aspect,
                    field: installed_field,
                },
            ) => {
                *entity == installed_entity
                    && *aspect == installed_aspect
                    && *field == installed_field
            }
            (
                Self::Link { relation, from, to },
                ApplicationOperationProgramTarget::Link {
                    relation: installed_relation,
                    from: installed_from,
                    to: installed_to,
                },
            )
            | (
                Self::Unlink { relation, from, to },
                ApplicationOperationProgramTarget::Unlink {
                    relation: installed_relation,
                    from: installed_from,
                    to: installed_to,
                },
            ) => *relation == installed_relation && *from == installed_from && *to == installed_to,
            (Self::Emit(expected), ApplicationOperationProgramTarget::Emit { effect }) => {
                *expected == effect
            }
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationSchemaAuthoringDenialKind {
    UnknownEntity,
    UnknownField,
    FieldFamilyMismatch,
    FieldValueTypeMismatch,
    FieldUnitMismatch,
    FieldNotWritable,
    FieldNotEqualityQueryable,
    UnknownOperation,
    OperationProvenanceMismatch,
    OperationInputTypeMismatch,
    UnknownEffect,
    EffectProvenanceMismatch,
    EffectPayloadTypeMismatch,
    UnknownRelation,
    OperationProgramNotInstalled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSchemaAuthoringDenial {
    kind: ApplicationSchemaAuthoringDenialKind,
    subject: String,
}

impl ApplicationSchemaAuthoringDenial {
    pub(crate) fn new(
        kind: ApplicationSchemaAuthoringDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub const fn kind(&self) -> ApplicationSchemaAuthoringDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for ApplicationSchemaAuthoringDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application schema authoring denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for ApplicationSchemaAuthoringDenial {}
