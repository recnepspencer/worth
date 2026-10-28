use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;

use super::WorthQueryOperationApplicationProjectionScope;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryOperationGraphReadScope {
    DerivedSource(super::WorthQueryOperationDerivedSourceReadScope),
    Components(super::WorthQueryOperationComponentReadScope),
    Entity(WorthQueryOperationEntityReadScope),
    NativeProjection(WorthQueryOperationApplicationProjectionScope),
    Relation(WorthQueryOperationRelationReadScope),
}

/// Installed entity-read meaning. Only application installation can construct it.
///
/// ```compile_fail
/// use worth_query_installation::facade::WorthQueryOperationEntityReadScope;
/// let _constructor = WorthQueryOperationEntityReadScope::new;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationEntityReadScope {
    schema: ApplicationSchemaBindingIdentity,
    entity: String,
}

impl WorthQueryOperationEntityReadScope {
    pub(crate) fn new(schema: ApplicationSchemaBindingIdentity, entity: String) -> Self {
        Self { schema, entity }
    }

    pub const fn schema(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }

    pub fn semantic_key(&self) -> &str {
        &self.entity
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationEntityReadScopeRef<'a> {
    schema: &'a ApplicationSchemaBindingIdentity,
    entity: &'a str,
}

impl<'a> WorthQueryOperationEntityReadScopeRef<'a> {
    pub(super) const fn new(schema: &'a ApplicationSchemaBindingIdentity, entity: &'a str) -> Self {
        Self { schema, entity }
    }

    pub const fn schema(self) -> &'a ApplicationSchemaBindingIdentity {
        self.schema
    }

    pub const fn semantic_key(self) -> &'a str {
        self.entity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationRelationReadScope {
    schema: ApplicationSchemaBindingIdentity,
    relation: String,
    from: String,
    to: String,
}

impl WorthQueryOperationRelationReadScope {
    pub(crate) fn new(
        schema: ApplicationSchemaBindingIdentity,
        relation: String,
        from: String,
        to: String,
    ) -> Self {
        Self {
            schema,
            relation,
            from,
            to,
        }
    }

    pub const fn schema(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }

    pub fn relation(&self) -> &str {
        &self.relation
    }

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn to(&self) -> &str {
        &self.to
    }
}
