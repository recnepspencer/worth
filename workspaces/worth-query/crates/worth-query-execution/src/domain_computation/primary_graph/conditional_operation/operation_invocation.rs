use std::sync::Arc;
use worth_query_execution_lineage::facade::ports::ApplicationInvariantProjectionAuthorityInternalPort as _;

use worth_query_installation::facade::{
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding, ApplicationSchema,
    ApplicationSchemaMember, WorthQueryTemporalIntentRevisionValue,
};

use crate::domain_computation::primary_graph::WorthQueryApplicationInvariantProjectionAuthority;

mod current_intent;
pub(in crate::domain_computation::primary_graph::conditional_operation) use current_intent::WorthQueryCurrentTemporalIntent;
mod due_field;
pub use due_field::{
    WorthQueryNoTemporalDueField, WorthQueryTemporalDueField, WorthQueryTemporalDueFieldWriter,
};
mod invoker_contract;
pub use invoker_contract::{
    WorthQueryTemporalInvocationFailure, WorthQueryTemporalInvocationFailureKind,
    WorthQueryTemporalOccurrenceOutcome, WorthQueryTemporalOperationInvoker,
};

/// Exact invariant and temporal-record mutation contract for wake re-entry.
pub struct WorthQueryTemporalOperationExecution<
    Schema,
    Operation,
    Input,
    Scope,
    Invoker,
    IntentEntity,
    IdentityField,
    IdentityValue,
    RevisionField,
    RevisionValue,
    LifecycleField,
    LifecycleValue,
    Authorization = super::WorthQueryPublicTemporalOperationAuthorization,
    DueField = WorthQueryNoTemporalDueField,
> {
    pub(super) invariant: Arc<WorthQueryApplicationInvariantProjectionAuthority<Schema>>,
    pub(super) invoker: Arc<Invoker>,
    pub(super) identity_field: IdentityField,
    pub(super) revision_field: RevisionField,
    pub(super) lifecycle_field: LifecycleField,
    pub(super) active_lifecycle: LifecycleValue,
    pub(super) completed_lifecycle: LifecycleValue,
    pub(super) authorization: Authorization,
    pub(super) due_field: DueField,
    marker: std::marker::PhantomData<
        fn(Input) -> (Operation, Scope, IntentEntity, IdentityValue, RevisionValue),
    >,
}

impl<
        Schema,
        Operation,
        Input,
        Scope,
        Invoker,
        IntentEntity,
        IdentityField,
        IdentityValue,
        RevisionField,
        RevisionValue,
        LifecycleField,
        LifecycleValue,
        Authorization,
    >
    WorthQueryTemporalOperationExecution<
        Schema,
        Operation,
        Input,
        Scope,
        Invoker,
        IntentEntity,
        IdentityField,
        IdentityValue,
        RevisionField,
        RevisionValue,
        LifecycleField,
        LifecycleValue,
        Authorization,
    >
where
    Invoker: WorthQueryTemporalOperationInvoker<Schema, Operation, Input, Scope>,
    IdentityField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
        Entity = IntentEntity,
        Value = IdentityValue,
    >,
    IdentityField::Binding: ApplicationReadableScalarValueBinding<Value = IdentityValue>,
    RevisionField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
        Entity = IntentEntity,
        Value = RevisionValue,
    >,
    RevisionField::Binding: ApplicationReadableScalarValueBinding<Value = RevisionValue>
        + WorthQueryTemporalIntentRevisionValue,
    LifecycleField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
        Entity = IntentEntity,
        Value = LifecycleValue,
    >,
    LifecycleField::Binding: ApplicationScalarValueBinding<Value = LifecycleValue>,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        invariant: Arc<WorthQueryApplicationInvariantProjectionAuthority<Schema>>,
        invoker: Invoker,
        identity_field: IdentityField,
        revision_field: RevisionField,
        lifecycle_field: LifecycleField,
        active_lifecycle: LifecycleValue,
        completed_lifecycle: LifecycleValue,
    ) -> Result<Self, &'static str>
    where
        Authorization: Default,
    {
        Self::with_authorization(
            invariant,
            invoker,
            identity_field,
            revision_field,
            lifecycle_field,
            active_lifecycle,
            completed_lifecycle,
            Authorization::default(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_authorization(
        invariant: Arc<WorthQueryApplicationInvariantProjectionAuthority<Schema>>,
        invoker: Invoker,
        identity_field: IdentityField,
        revision_field: RevisionField,
        lifecycle_field: LifecycleField,
        active_lifecycle: LifecycleValue,
        completed_lifecycle: LifecycleValue,
        authorization: Authorization,
    ) -> Result<Self, &'static str> {
        let identity = Invoker::SEMANTIC_IDENTITY;
        if identity.is_empty()
            || identity.trim() != identity
            || identity.chars().any(char::is_whitespace)
        {
            return Err("invalid-temporal-operation-invoker-identity");
        }
        Ok(Self {
            invariant,
            invoker: Arc::new(invoker),
            identity_field,
            revision_field,
            lifecycle_field,
            active_lifecycle,
            completed_lifecycle,
            authorization,
            due_field: WorthQueryNoTemporalDueField,
            marker: std::marker::PhantomData,
        })
    }

    /// Configures the authoritative due-coordinate field written atomically
    /// when [`WorthQueryTemporalOperationInvoker::next_occurrence`] reschedules
    /// a recurring timed intent.
    pub fn with_due_field<Field>(
        self,
        due_field: Field,
    ) -> WorthQueryTemporalOperationExecution<
        Schema,
        Operation,
        Input,
        Scope,
        Invoker,
        IntentEntity,
        IdentityField,
        IdentityValue,
        RevisionField,
        RevisionValue,
        LifecycleField,
        LifecycleValue,
        Authorization,
        WorthQueryTemporalDueField<Field>,
    >
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
                Entity = IntentEntity,
            > + Copy
            + Send
            + Sync
            + 'static,
        Field::Binding: ApplicationScalarValueBinding<Value = Field::Value>
            + worth_query_installation::facade::WorthQueryTemporalIntentDueValue,
    {
        WorthQueryTemporalOperationExecution {
            invariant: self.invariant,
            invoker: self.invoker,
            identity_field: self.identity_field,
            revision_field: self.revision_field,
            lifecycle_field: self.lifecycle_field,
            active_lifecycle: self.active_lifecycle,
            completed_lifecycle: self.completed_lifecycle,
            authorization: self.authorization,
            due_field: WorthQueryTemporalDueField(due_field),
            marker: std::marker::PhantomData,
        }
    }
}

impl<
        Schema,
        Operation,
        Input,
        Scope,
        Invoker,
        IntentEntity,
        IdentityField,
        IdentityValue,
        RevisionField,
        RevisionValue,
        LifecycleField,
        LifecycleValue,
        Authorization,
        DueField,
    >
    WorthQueryTemporalOperationExecution<
        Schema,
        Operation,
        Input,
        Scope,
        Invoker,
        IntentEntity,
        IdentityField,
        IdentityValue,
        RevisionField,
        RevisionValue,
        LifecycleField,
        LifecycleValue,
        Authorization,
        DueField,
    >
where
    Invoker: WorthQueryTemporalOperationInvoker<Schema, Operation, Input, Scope>,
    IdentityField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
        Entity = IntentEntity,
        Value = IdentityValue,
    >,
    IdentityField::Binding: ApplicationReadableScalarValueBinding<Value = IdentityValue>,
    RevisionField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
        Entity = IntentEntity,
        Value = RevisionValue,
    >,
    RevisionField::Binding: ApplicationReadableScalarValueBinding<Value = RevisionValue>
        + WorthQueryTemporalIntentRevisionValue,
    LifecycleField: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
        Entity = IntentEntity,
        Value = LifecycleValue,
    >,
    LifecycleField::Binding: ApplicationScalarValueBinding<Value = LifecycleValue>,
{
    pub fn invoker_identity(&self) -> &'static str {
        Invoker::SEMANTIC_IDENTITY
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_authorization_and_due_field(
        invariant: Arc<WorthQueryApplicationInvariantProjectionAuthority<Schema>>,
        invoker: Invoker,
        identity_field: IdentityField,
        revision_field: RevisionField,
        lifecycle_field: LifecycleField,
        active_lifecycle: LifecycleValue,
        completed_lifecycle: LifecycleValue,
        authorization: Authorization,
        due_field: DueField,
    ) -> Result<Self, &'static str> {
        let identity = Invoker::SEMANTIC_IDENTITY;
        if identity.is_empty()
            || identity.trim() != identity
            || identity.chars().any(char::is_whitespace)
        {
            return Err("invalid-temporal-operation-invoker-identity");
        }
        Ok(Self {
            invariant,
            invoker: Arc::new(invoker),
            identity_field,
            revision_field,
            lifecycle_field,
            active_lifecycle,
            completed_lifecycle,
            authorization,
            due_field,
            marker: std::marker::PhantomData,
        })
    }

    pub(super) fn validate_publication(
        &self,
        publication: &crate::domain_computation::primary_graph::application_runtime::installation::ApplicationRuntimePublication<Schema>,
    ) -> Result<(), String>
    where
        Schema: ApplicationSchema,
        DueField: WorthQueryTemporalDueFieldWriter<Schema, IntentEntity>,
    {
        if !self.invariant.belongs_to_installation(
            publication.bootstrap.runtime_authority,
            &publication.installed_schema.binding_identity(),
        ) {
            return Err("temporal invariant authority belongs to another installation".into());
        }
        let members = publication
            .installed_schema
            .installed_declaration()
            .members();
        for field in [
            field_coordinates(&self.identity_field),
            field_coordinates(&self.revision_field),
            field_coordinates(&self.lifecycle_field),
        ] {
            if !members.iter().any(|member| field.matches(member)) {
                return Err(format!(
                    "temporal operation field `{}` is not installed",
                    field.field
                ));
            }
        }
        self.due_field.validate_installed(members)?;
        Ok(())
    }
}

struct TemporalFieldCoordinates<'field> {
    entity: &'field str,
    aspect: &'field str,
    field: &'field str,
    scalar_family: worth_foundational::facade::ScalarAspectType,
}

impl TemporalFieldCoordinates<'_> {
    fn matches(&self, member: &ApplicationSchemaMember) -> bool {
        matches!(member, ApplicationSchemaMember::Field {
            entity,
            aspect,
            field,
            scalar_family,
            ..
        } if entity == self.entity
            && aspect == self.aspect
            && field == self.field
            && *scalar_family == self.scalar_family)
    }
}

fn field_coordinates<Entity, Aspect, Field, Value>(field: &Field) -> TemporalFieldCoordinates<'_>
where
    Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
        Entity = Entity,
        Aspect = Aspect,
        Value = Value,
    >,
{
    TemporalFieldCoordinates {
        entity: field.entity(),
        aspect: field.aspect(),
        field: field.field(),
        scalar_family: field.scalar_family(),
    }
}
