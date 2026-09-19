use std::sync::Arc;

use worth_query_decl::facade::application_schema::ApplicationFieldRef;

use super::{
    CollectionItemKeyField, CollectionItemStatusField, IdentityIdField, MeasurementValueField,
    QueryRevisionValueField, QueryTextStatusField, SizeValueField, WorthUiApplicationSchema,
    WorthUiProjectionField,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiProjectionFieldRequirementError {
    Empty,
    SurroundingWhitespace,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct UiProjectionFieldRequirement {
    authority: UiProjectionFieldAuthority,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum UiProjectionFieldAuthority {
    WorthUi(WorthUiProjectionField),
    Diagnostic(Arc<str>),
}

impl UiProjectionFieldRequirement {
    /// Diagnostic name constructor reserved for invalid-input and fixture claims.
    pub fn declared(name: impl Into<String>) -> Result<Self, UiProjectionFieldRequirementError> {
        let name = name.into();
        if name.is_empty() {
            return Err(UiProjectionFieldRequirementError::Empty);
        }
        if name.trim() != name {
            return Err(UiProjectionFieldRequirementError::SurroundingWhitespace);
        }
        Ok(Self {
            authority: UiProjectionFieldAuthority::Diagnostic(Arc::from(name)),
        })
    }

    pub fn declared_name(&self) -> &str {
        match &self.authority {
            UiProjectionFieldAuthority::WorthUi(field) => field.native_key(),
            UiProjectionFieldAuthority::Diagnostic(name) => name,
        }
    }

    pub fn typed_field(&self) -> Option<WorthUiProjectionField> {
        match self.authority {
            UiProjectionFieldAuthority::WorthUi(field) => Some(field),
            UiProjectionFieldAuthority::Diagnostic(_) => None,
        }
    }

    pub(crate) fn native_key(&self) -> &str {
        self.declared_name()
    }

    pub(crate) fn collection_contract_key(&self) -> &str {
        match &self.authority {
            UiProjectionFieldAuthority::WorthUi(field) => field.collection_contract_key(),
            UiProjectionFieldAuthority::Diagnostic(name) => name,
        }
    }

    pub fn query_text_status() -> Self {
        Self::from_worth_ui_field(QueryTextStatusField::reference())
    }

    pub fn measurement_value() -> Self {
        Self::from_worth_ui_field(MeasurementValueField::reference())
    }

    pub fn size_value() -> Self {
        Self::from_worth_ui_field(SizeValueField::reference())
    }

    pub fn identity_id() -> Self {
        Self::from_worth_ui_field(IdentityIdField::reference())
    }

    pub fn query_revision() -> Self {
        Self::from_worth_ui_field(QueryRevisionValueField::reference())
    }

    pub fn collection_item_status() -> Self {
        Self::from_worth_ui_field(CollectionItemStatusField::reference())
    }

    pub fn collection_item_key() -> Self {
        Self::from_worth_ui_field(CollectionItemKeyField::reference())
    }

    /// Bind a Worth UI schema field. Names remain diagnostic only.
    ///
    /// Cross-schema fields are unrepresentable:
    ///
    /// ```compile_fail
    /// use worth_query_decl::facade::{worth_query_application_schema, worth_query_aspect, worth_query_entity, worth_query_field};
    /// use worth_ui_query_binding::UiProjectionFieldRequirement;
    /// worth_query_application_schema! {
    ///     pub schema OtherSchema {
    ///         owner: other,
    ///         version: (1, 0),
    ///         members: |schema| { schema.entity(OtherRecord::reference()).aspect(OtherRecord::reference(), OtherAspect::reference()).field(OtherRecord::reference(), OtherField::reference()) }
    ///     }
    /// }
    /// worth_query_entity!(pub OtherRecord for OtherSchema);
    /// worth_query_aspect!(pub OtherAspect for OtherSchema, OtherRecord; identity = AspectIdentity(0x9161105b), revision = AspectContractRevision(1),);
    /// worth_query_field!(pub OtherField for OtherSchema, OtherRecord, OtherAspect: String => worth_query_decl::facade::application_schema::StringApplicationValueBinding, read_only, equality);
    /// let _ = UiProjectionFieldRequirement::from_worth_ui_field(OtherField::reference());
    /// ```
    ///
    /// Cross-aspect fields are unrepresentable:
    ///
    /// ```compile_fail
    /// use worth_ui_query_binding::{IdentityIdField, QueryTextStatusField, UiProjectionFieldRequirement};
    /// fn bind_status(
    ///     field: worth_query_decl::facade::application_schema::ApplicationFieldRef<
    ///         worth_ui_query_binding::WorthUiApplicationSchema,
    ///         worth_ui_query_binding::WorthUiRecord,
    ///         worth_ui_query_binding::QueryTextAspect,
    ///         QueryTextStatusField,
    ///         String,
    ///     >,
    /// ) -> UiProjectionFieldRequirement {
    ///     UiProjectionFieldRequirement::from_worth_ui_field(field)
    /// }
    /// let _ = bind_status(IdentityIdField::reference());
    /// ```
    ///
    /// Wrong value types are unrepresentable:
    ///
    /// ```compile_fail
    /// use worth_ui_query_binding::{QueryTextStatusField, UiProjectionFieldRequirement};
    /// fn bind_f32(
    ///     field: worth_query_decl::facade::application_schema::ApplicationFieldRef<
    ///         worth_ui_query_binding::WorthUiApplicationSchema,
    ///         worth_ui_query_binding::WorthUiRecord,
    ///         worth_ui_query_binding::QueryTextAspect,
    ///         QueryTextStatusField,
    ///         f32,
    ///     >,
    /// ) -> UiProjectionFieldRequirement {
    ///     UiProjectionFieldRequirement::from_worth_ui_field(field)
    /// }
    /// let _ = bind_f32(QueryTextStatusField::reference());
    /// ```
    pub fn from_worth_ui_field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        field: ApplicationFieldRef<
            WorthUiApplicationSchema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            Equality,
            Unit,
        >,
    ) -> Self
    where
        Field: super::WorthUiNativeField,
    {
        let _typed_authority = field;
        Self {
            authority: UiProjectionFieldAuthority::WorthUi(Field::FIELD),
        }
    }
}
