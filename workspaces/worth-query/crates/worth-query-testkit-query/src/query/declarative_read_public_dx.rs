pub mod read_journeys {
    pub use worth_query::facade::read::{
        current, declare, AspectFieldSelector, AspectName, AuthoredResultShapeField,
        DetailQueryBuilder, DetailResultShapeBuilder, FieldName, QuerySchemaView,
        QueryScopeDescriptor, QueryTemplateDescriptor, RootEntityKey, ScalarAspectType,
        SchemaFieldView, TemplateBindingSet, TemplateParameterSlot, WorthQueryReadContextKind,
    };

    pub fn identity_query() -> worth_query::facade::read::DetailAuthoredQuery {
        DetailQueryBuilder::new(RootEntityKey::new("user").expect("root should build"))
            .project(identity_selector())
            .build()
            .expect("query should build")
    }

    pub fn composition_shape() -> worth_query::facade::read::DetailAuthoredResultShape {
        DetailResultShapeBuilder::new()
            .field(identity_field())
            .field(name_field())
            .build()
            .expect("shape should build")
    }

    pub fn composition_schema() -> QuerySchemaView {
        QuerySchemaView::new(
            "public-composition",
            [
                SchemaFieldView::new(
                    AspectName::new("identity").expect("aspect should build"),
                    FieldName::new("id").expect("field should build"),
                    ScalarAspectType::String,
                ),
                SchemaFieldView::new(
                    AspectName::new("profile").expect("aspect should build"),
                    FieldName::new("display_name").expect("field should build"),
                    ScalarAspectType::String,
                ),
            ],
            [],
        )
    }

    pub fn identity_selector() -> AspectFieldSelector {
        AspectFieldSelector::new("identity", "id").expect("selector should build")
    }

    pub fn name_selector() -> AspectFieldSelector {
        AspectFieldSelector::new("profile", "display_name").expect("selector should build")
    }

    pub fn identity_field() -> AuthoredResultShapeField {
        AuthoredResultShapeField::new("identity", "id", "id").expect("field should build")
    }

    pub fn name_field() -> AuthoredResultShapeField {
        AuthoredResultShapeField::new("profile", "display_name", "display_name")
            .expect("field should build")
    }
}

pub mod aggregate_journey {
    pub use worth_query::facade::aggregate::{
        current, declare, AspectFieldSelector, AspectName, AuthoredResultShapeField, FieldName,
        QuerySchemaView, ScalarAspectType, SchemaFieldView, WorthQueryReadContextKind,
    };
}

pub mod live_declaration_prefix {
    pub use worth_query::facade::live::{
        current, declare, AspectFieldSelector, AspectName, AuthoredResultShapeField, FieldName,
        QuerySchemaView, ScalarAspectType, SchemaFieldView, WorthQueryReadContextKind,
    };
}
