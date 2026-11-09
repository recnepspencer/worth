//! Typed creation authoring. A created entity is not the admitted scope.

use std::marker::PhantomData;

use crate::application_schema::{ApplicationEntityRef, OperationCreates};

use super::{
    ApplicationOperationBody, ApplicationOperationBodyAuthoringDenial,
    ApplicationOperationBodyBuilder, ApplicationOperationBodyCreatedField,
    ApplicationOperationBodyCreation,
};

pub struct ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity> {
    pub(super) parent: ApplicationOperationBodyBuilder<Schema, Operation, Input>,
    entity: String,
    pub(super) key: String,
    pub(super) fields: Vec<ApplicationOperationBodyCreatedField>,
    pub(super) links: Vec<super::ApplicationOperationBodyCreationLink>,
    pub(super) signing: Option<super::ApplicationOperationBodySigningSpec>,
    pub(super) signing_lifecycle: Option<super::ApplicationOperationBodySigningLifecycleSpec>,
    pub(super) subject_impact_bound: Option<std::num::NonZeroUsize>,
    pub(super) enrollment: Option<super::ApplicationOperationBodyEnrollment>,
    pub(super) reuse_current_if_unchanged: bool,
    pub(super) peers: Vec<ApplicationOperationBodyCreation>,
    pub(super) path_links: Vec<super::ApplicationOperationBodyCreationPathLink>,
    pub(super) children: Vec<super::ApplicationOperationBodyChildCreation>,
    marker: PhantomData<fn() -> Entity>,
}

impl<Schema, Operation, Input> ApplicationOperationBodyBuilder<Schema, Operation, Input> {
    /// Creates in the admitted existing scope's context. The key is a
    /// candidate-local symbolic slot, not a business or idempotency identity.
    pub fn create_in_scope<Entity>(
        self,
        entity: ApplicationEntityRef<Schema, Entity>,
        key: &str,
    ) -> Result<
        ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>,
        ApplicationOperationBodyAuthoringDenial,
    >
    where
        Entity: OperationCreates<Operation>,
    {
        if key.is_empty() || key.trim() != key {
            return Err(ApplicationOperationBodyAuthoringDenial::InvalidCreationKey);
        }
        Ok(ApplicationOperationBodyCreationBuilder {
            parent: self,
            entity: entity.name().to_owned(),
            key: key.to_owned(),
            fields: Vec::new(),
            links: Vec::new(),
            signing: None,
            signing_lifecycle: None,
            subject_impact_bound: None,
            enrollment: None,
            reuse_current_if_unchanged: false,
            children: Vec::new(),
            peers: Vec::new(),
            path_links: Vec::new(),
            marker: PhantomData,
        })
    }
}

impl<Schema, Operation, Input, Entity>
    ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
{
    pub fn initialize<
        Selection: crate::application_schema::fragment::RecordSelection<Entity = Entity>,
    >(
        mut self,
        field: Selection,
        value: Selection::Value,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial> {
        self.fields
            .push(super::initialization::literal(field, value)?);
        Ok(self)
    }

    pub fn initialize_input<
        Selection: crate::application_schema::fragment::RecordSelection<Entity = Entity>,
    >(
        mut self,
        field: Selection,
        member: &str,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial> {
        self.fields
            .push(super::initialization::input(field, member, false)?);
        Ok(self)
    }

    /// Initializes an optional created field from a serialized Option. None
    /// leaves it absent; Some initializes it through the installed scalar codec.
    pub fn initialize_optional_input<
        Selection: crate::application_schema::fragment::RecordSelection<Entity = Entity>,
    >(
        mut self,
        field: Selection,
        member: &str,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial> {
        self.fields
            .push(super::initialization::input(field, member, true)?);
        Ok(self)
    }

    /// Copies the scope's tracked preimage into one created field. Both fields
    /// share the same value, unit and exact codec; the host supplies no value.
    pub fn initialize_from_scope<Source, Field>(
        mut self,
        source: Source,
        field: Field,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial>
    where
        Source: crate::application_schema::fragment::RecordSelection,
        Field: crate::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Value = Source::Value,
            Binding = Source::Binding,
        >,
    {
        let source = super::record_authoring::resolve(source)?;
        let field = super::record_authoring::resolve(field)?;
        self.fields
            .push(ApplicationOperationBodyCreatedField::ScopeField {
                field: field.locus().clone(),
                binding: field.recipe().binding_identity().clone(),
                scalar_family: field.recipe().scalar_family(),
                source: source.locus().clone(),
            });
        Ok(self)
    }

    pub fn finish(
        self,
    ) -> Result<
        ApplicationOperationBody<Schema, Operation, Input>,
        ApplicationOperationBodyAuthoringDenial,
    > {
        let creation = ApplicationOperationBodyCreation {
            entity: self.entity,
            key: self.key,
            fields: self.fields,
            links: self.links,
            signing: self.signing,
            signing_lifecycle: self.signing_lifecycle,
            subject_impact_bound: self.subject_impact_bound,
            enrollment: self.enrollment,
            reuse_current_if_unchanged: self.reuse_current_if_unchanged,
            children: self.children,
            peers: self.peers,
            path_links: self.path_links,
        };
        super::classified_capture::validate_creation(&creation)?;
        if !super::signing_lifecycle::validation::validate_creation(&creation) {
            return Err(ApplicationOperationBodyAuthoringDenial::InvalidSigningLifecycle);
        }
        let body = self.parent.finish_with_creation(Some(creation))?;
        if !super::enrollment::valid(&body.program) {
            return Err(ApplicationOperationBodyAuthoringDenial::InvalidWorkflowEnrollment);
        }
        if !super::signing_lifecycle::validation::validate_subject_writes(&body.program) {
            return Err(ApplicationOperationBodyAuthoringDenial::InvalidSigningLifecycle);
        }
        let scope = body
            .program
            .scope_entity()
            .expect("completed body has a scope");
        let mut fields = std::collections::BTreeSet::new();
        if body
            .program
            .creation()
            .expect("creation attached")
            .links()
            .iter()
            .any(|link| {
                (matches!(
                    link.source,
                    super::ApplicationOperationBodyCreationLinkSource::Scope
                ) && link.scope_entity() != scope)
                    || link.created_entity()
                        != body.program.creation().expect("creation attached").entity()
            })
        {
            return Err(ApplicationOperationBodyAuthoringDenial::InvalidCreationLink);
        }
        for creation in body
            .program
            .creation()
            .into_iter()
            .flat_map(|c| c.creation_roots())
        {
            fields.clear();
            for step in creation.fields() {
                if !fields.insert(step.field()) {
                    return Err(ApplicationOperationBodyAuthoringDenial::DuplicateAssignment);
                }
                let foreign_source = if let Some(path) = step.path_source() {
                    !path.has_supported_shape() || path.path.start_entity() != scope
                } else {
                    match step.related_source() {
                        Some(source) => source.from != scope || source.field.entity() != source.to,
                        None => step
                            .source_field()
                            .is_some_and(|source| source.entity() != scope),
                    }
                };
                if foreign_source {
                    return Err(ApplicationOperationBodyAuthoringDenial::ForeignSourceEntity);
                }
            }
        }
        Ok(body)
    }
}
