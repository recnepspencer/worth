use std::sync::Arc;

use worth_query_decl::facade::application_schema::fragment::RecordSelection;

use super::{
    CollectionItemKeyField, CollectionItemStatusField, IdentityIdField, MeasurementValueField,
    QueryRevisionValueField, QueryTextStatusField, SizeValueField, WorthUiProjectionField,
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
        Self::from_worth_ui_field(QueryTextStatusField::at("QueryTextStatusField"))
    }

    pub fn measurement_value() -> Self {
        Self::from_worth_ui_field(MeasurementValueField::at("MeasurementValueField"))
    }

    pub fn size_value() -> Self {
        Self::from_worth_ui_field(SizeValueField::at("SizeValueField"))
    }

    pub fn identity_id() -> Self {
        Self::from_worth_ui_field(IdentityIdField::at("IdentityIdField"))
    }

    pub fn query_revision() -> Self {
        Self::from_worth_ui_field(QueryRevisionValueField::at("QueryRevisionValueField"))
    }

    pub fn collection_item_status() -> Self {
        Self::from_worth_ui_field(CollectionItemStatusField::at("CollectionItemStatusField"))
    }

    pub fn collection_item_key() -> Self {
        Self::from_worth_ui_field(CollectionItemKeyField::at("CollectionItemKeyField"))
    }

    /// Bind a Worth UI schema field. Names remain diagnostic only.
    ///
    /// Foreign record fields cannot acquire the UI's sealed projection contract:
    ///
    /// ```compile_fail
    /// use worth_query_decl::facade::application_schema::{*, fragment::*};
    /// use worth_ui_query_binding::UiProjectionFieldRequirement;
    /// struct Other;
    /// impl Fragment for Other { const NAME: &'static str = "other"; type Bindings = Set; type Members = (Item,); }
    /// struct Item;
    /// impl Entity for Item { const NAME: &'static str = "item"; type Aspects = (Facts,); type Relations = (); }
    /// struct Facts;
    /// impl Aspect for Facts { const NAME: &'static str = "facts"; type Row = u64; }
    /// struct Set;
    /// impl BindingSet for Set {
    ///     type Work<'a> = ValueDecodeAdmission<'a>;
    ///     fn with_admission<T>(work: &mut Self::Work<'_>, run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T) -> T { run(work) }
    /// }
    /// impl Scalar<u64> for Set { type Binding = U64ApplicationValueBinding; }
    /// let field = FieldSelection::<Other, Item, Facts, u64>::at("value");
    /// let _ = UiProjectionFieldRequirement::from_worth_ui_field(field);
    /// ```
    ///
    /// Aspect identity remains part of the record handle's type:
    ///
    /// ```compile_fail
    /// use worth_ui_query_binding::{IdentityIdField, QueryTextStatusField, UiProjectionFieldRequirement};
    /// fn bind_status(field: QueryTextStatusField) -> UiProjectionFieldRequirement {
    ///     UiProjectionFieldRequirement::from_worth_ui_field(field)
    /// }
    /// let _ = bind_status(IdentityIdField::at("IdentityIdField"));
    /// ```
    ///
    /// Value types remain checked:
    ///
    /// ```compile_fail
    /// use worth_query_decl::facade::application_schema::fragment::RecordSelection;
    /// use worth_ui_query_binding::{QueryTextStatusField, WorthUiNativeField, UiProjectionFieldRequirement};
    /// fn bind_f32<F: RecordSelection<Value=f32> + WorthUiNativeField>(field: F) -> UiProjectionFieldRequirement {
    ///     UiProjectionFieldRequirement::from_worth_ui_field(field)
    /// }
    /// let _ = bind_f32(QueryTextStatusField::at("QueryTextStatusField"));
    /// ```
    pub fn from_worth_ui_field<Field>(field: Field) -> Self
    where
        Field: RecordSelection + super::WorthUiNativeField,
    {
        let _typed_authority = field;
        Self {
            authority: UiProjectionFieldAuthority::WorthUi(Field::FIELD),
        }
    }
}
