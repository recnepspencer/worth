use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationReadableScalarValueBinding,
    ApplicationScalarValueBinding, ApplicationSchema, ApplicationSchemaMember,
    DeclaredApplicationFieldValue, EqualityPredicate, OperationWrites,
    WorthQueryTemporalIntentRevisionValue, WritableCapability, WritePosture,
};

use crate::domain_computation::primary_graph::WorthQueryApplicationInvariantProjectionAuthority;

mod current_intent;
pub(in crate::domain_computation::primary_graph::conditional_operation) use current_intent::WorthQueryCurrentTemporalIntent;
mod invoker_contract;
pub use invoker_contract::{
    WorthQueryTemporalInvocationFailure, WorthQueryTemporalInvocationFailureKind,
    WorthQueryTemporalOperationInvoker,
};

/// Exact invariant and temporal-record mutation contract for wake re-entry.
pub struct WorthQueryTemporalOperationExecution<
    Schema,
    Operation,
    Input,
    Scope,
    Invoker,
    IntentEntity,
    IdentityAspect,
    IdentityField,
    IdentityValue,
    IdentityWrite,
    IdentityUnit,
    RevisionAspect,
    RevisionField,
    RevisionValue,
    RevisionWrite,
    RevisionEquality,
    RevisionUnit,
    LifecycleAspect,
    LifecycleField,
    LifecycleValue,
    LifecycleWrite,
    LifecycleEquality,
    LifecycleUnit,
    Authorization = super::WorthQueryPublicTemporalOperationAuthorization,
> {
    pub(super) invariant: Arc<WorthQueryApplicationInvariantProjectionAuthority<Schema>>,
    pub(super) invoker: Arc<Invoker>,
    pub(super) identity_field: ApplicationFieldRef<
        Schema,
        IntentEntity,
        IdentityAspect,
        IdentityField,
        IdentityValue,
        IdentityWrite,
        EqualityPredicate,
        IdentityUnit,
    >,
    pub(super) revision_field: ApplicationFieldRef<
        Schema,
        IntentEntity,
        RevisionAspect,
        RevisionField,
        RevisionValue,
        RevisionWrite,
        RevisionEquality,
        RevisionUnit,
    >,
    pub(super) lifecycle_field: ApplicationFieldRef<
        Schema,
        IntentEntity,
        LifecycleAspect,
        LifecycleField,
        LifecycleValue,
        LifecycleWrite,
        LifecycleEquality,
        LifecycleUnit,
    >,
    pub(super) active_lifecycle: LifecycleValue,
    pub(super) completed_lifecycle: LifecycleValue,
    pub(super) authorization: Authorization,
    marker: std::marker::PhantomData<fn(Input) -> (Operation, Scope)>,
}

impl<
        Schema,
        Operation,
        Input,
        Scope,
        Invoker,
        IntentEntity,
        IdentityAspect,
        IdentityField,
        IdentityValue,
        IdentityWrite,
        IdentityUnit,
        RevisionAspect,
        RevisionField,
        RevisionValue,
        RevisionWrite,
        RevisionEquality,
        RevisionUnit,
        LifecycleAspect,
        LifecycleField,
        LifecycleValue,
        LifecycleWrite,
        LifecycleEquality,
        LifecycleUnit,
        Authorization,
    >
    WorthQueryTemporalOperationExecution<
        Schema,
        Operation,
        Input,
        Scope,
        Invoker,
        IntentEntity,
        IdentityAspect,
        IdentityField,
        IdentityValue,
        IdentityWrite,
        IdentityUnit,
        RevisionAspect,
        RevisionField,
        RevisionValue,
        RevisionWrite,
        RevisionEquality,
        RevisionUnit,
        LifecycleAspect,
        LifecycleField,
        LifecycleValue,
        LifecycleWrite,
        LifecycleEquality,
        LifecycleUnit,
        Authorization,
    >
where
    Invoker: WorthQueryTemporalOperationInvoker<Schema, Operation, Input, Scope>,
    IdentityField: DeclaredApplicationFieldValue<Value = IdentityValue>,
    IdentityField::Binding: ApplicationReadableScalarValueBinding<Value = IdentityValue>,
    IdentityWrite: WritePosture,
    IdentityUnit: ApplicationFieldUnit,
    RevisionField:
        OperationWrites<Operation> + DeclaredApplicationFieldValue<Value = RevisionValue>,
    RevisionField::Binding: ApplicationReadableScalarValueBinding<Value = RevisionValue>
        + WorthQueryTemporalIntentRevisionValue,
    RevisionWrite: WritableCapability,
    RevisionUnit: ApplicationFieldUnit,
    LifecycleField: OperationWrites<Operation>,
    LifecycleField: DeclaredApplicationFieldValue<Value = LifecycleValue>,
    LifecycleField::Binding: ApplicationScalarValueBinding<Value = LifecycleValue>,
    LifecycleWrite: WritableCapability,
    LifecycleUnit: ApplicationFieldUnit,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        invariant: Arc<WorthQueryApplicationInvariantProjectionAuthority<Schema>>,
        invoker: Invoker,
        identity_field: ApplicationFieldRef<
            Schema,
            IntentEntity,
            IdentityAspect,
            IdentityField,
            IdentityValue,
            IdentityWrite,
            EqualityPredicate,
            IdentityUnit,
        >,
        revision_field: ApplicationFieldRef<
            Schema,
            IntentEntity,
            RevisionAspect,
            RevisionField,
            RevisionValue,
            RevisionWrite,
            RevisionEquality,
            RevisionUnit,
        >,
        lifecycle_field: ApplicationFieldRef<
            Schema,
            IntentEntity,
            LifecycleAspect,
            LifecycleField,
            LifecycleValue,
            LifecycleWrite,
            LifecycleEquality,
            LifecycleUnit,
        >,
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
        identity_field: ApplicationFieldRef<
            Schema,
            IntentEntity,
            IdentityAspect,
            IdentityField,
            IdentityValue,
            IdentityWrite,
            EqualityPredicate,
            IdentityUnit,
        >,
        revision_field: ApplicationFieldRef<
            Schema,
            IntentEntity,
            RevisionAspect,
            RevisionField,
            RevisionValue,
            RevisionWrite,
            RevisionEquality,
            RevisionUnit,
        >,
        lifecycle_field: ApplicationFieldRef<
            Schema,
            IntentEntity,
            LifecycleAspect,
            LifecycleField,
            LifecycleValue,
            LifecycleWrite,
            LifecycleEquality,
            LifecycleUnit,
        >,
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
            marker: std::marker::PhantomData,
        })
    }

    pub fn invoker_identity(&self) -> &'static str {
        Invoker::SEMANTIC_IDENTITY
    }

    pub(super) fn validate_publication(
        &self,
        publication: &crate::domain_computation::primary_graph::application_runtime::installation::ApplicationRuntimePublication<Schema>,
    ) -> Result<(), String>
    where
        Schema: ApplicationSchema,
    {
        if !self.invariant.belongs_to_installation(
            publication.bootstrap.runtime_authority,
            &publication.installed_schema.binding_identity(),
        ) {
            return Err("temporal invariant authority belongs to another installation".into());
        }
        for field in [
            field_coordinates(&self.identity_field),
            field_coordinates(&self.revision_field),
            field_coordinates(&self.lifecycle_field),
        ] {
            if !publication
                .installed_schema
                .installed_declaration()
                .members()
                .iter()
                .any(|member| field.matches(member))
            {
                return Err(format!(
                    "temporal operation field `{}` is not installed",
                    field.field
                ));
            }
        }
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

fn field_coordinates<Schema, Entity, Aspect, Field, Value, Write, Predicate, Unit>(
    field: &ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Predicate, Unit>,
) -> TemporalFieldCoordinates<'_>
where
    Field: DeclaredApplicationFieldValue<Value = Value>,
    Unit: ApplicationFieldUnit,
{
    TemporalFieldCoordinates {
        entity: field.entity(),
        aspect: field.aspect(),
        field: field.field(),
        scalar_family: field.scalar_family(),
    }
}
