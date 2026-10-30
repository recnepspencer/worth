use std::cmp::Ordering;
use std::sync::Arc;

use worth_foundational::facade::{AspectContract, CanonicalBasisReadyArtifact, CanonicalFieldPath};
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryOperationTouchScope {
    CreateEntity(WorthQueryOperationEntityTouchScope),
    DeleteEntity(WorthQueryOperationEntityTouchScope),
    WriteField(WorthQueryOperationFieldTouchScope),
    LinkRelation(WorthQueryOperationRelationTouchScope),
    UnlinkRelation(WorthQueryOperationRelationTouchScope),
    InitializeRelationField(super::WorthQueryOperationRelationFieldInitializationScope),
    DeclaredDomain(WorthQueryDeclaredDomainTouchScopeIdentity),
}

impl WorthQueryOperationTouchScope {
    #[expect(
        clippy::disallowed_methods,
        reason = "Comparison uses declared semantic keys; type names label diagnostics only"
    )]
    pub(crate) fn canonical_order(left: &Self, right: &Self) -> Ordering {
        touch_rank(left)
            .cmp(&touch_rank(right))
            .then_with(|| match (left, right) {
                (
                    Self::CreateEntity(left) | Self::DeleteEntity(left),
                    Self::CreateEntity(right) | Self::DeleteEntity(right),
                ) => compare_binding(left.schema(), right.schema())
                    .then_with(|| left.entity().cmp(right.entity())),
                (Self::WriteField(left), Self::WriteField(right)) => {
                    compare_binding(left.schema(), right.schema())
                        .then_with(|| left.entity().cmp(right.entity()))
                        .then_with(|| {
                            left.contract()
                                .key()
                                .as_str()
                                .cmp(right.contract().key().as_str())
                        })
                        .then_with(|| left.field_path().cmp(right.field_path()))
                }
                (
                    Self::LinkRelation(left) | Self::UnlinkRelation(left),
                    Self::LinkRelation(right) | Self::UnlinkRelation(right),
                ) => compare_binding(left.schema(), right.schema())
                    .then_with(|| left.relation().cmp(right.relation()))
                    .then_with(|| left.from().cmp(right.from()))
                    .then_with(|| left.to().cmp(right.to())),
                (Self::DeclaredDomain(left), Self::DeclaredDomain(right)) => left.cmp(right),
                (Self::InitializeRelationField(left), Self::InitializeRelationField(right)) => {
                    compare_binding(left.relation().schema(), right.relation().schema())
                        .then_with(|| left.relation().relation().cmp(right.relation().relation()))
                        .then_with(|| left.relation().from().cmp(right.relation().from()))
                        .then_with(|| left.relation().to().cmp(right.relation().to()))
                        .then_with(|| left.contract().key().cmp(right.contract().key()))
                        .then_with(|| left.field_path().cmp(right.field_path()))
                }
                _ => Ordering::Equal,
            })
    }
}

fn touch_rank(scope: &WorthQueryOperationTouchScope) -> u8 {
    match scope {
        WorthQueryOperationTouchScope::CreateEntity(_) => 0,
        WorthQueryOperationTouchScope::DeleteEntity(_) => 1,
        WorthQueryOperationTouchScope::WriteField(_) => 2,
        WorthQueryOperationTouchScope::LinkRelation(_) => 3,
        WorthQueryOperationTouchScope::UnlinkRelation(_) => 4,
        WorthQueryOperationTouchScope::DeclaredDomain(_) => 5,
        WorthQueryOperationTouchScope::InitializeRelationField(_) => 6,
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "Comparison uses declared semantic keys; type names label diagnostics only"
)]
fn compare_binding(
    left: &ApplicationSchemaBindingIdentity,
    right: &ApplicationSchemaBindingIdentity,
) -> Ordering {
    left.runtime_ordinal()
        .cmp(&right.runtime_ordinal())
        .then_with(|| left.generation().cmp(&right.generation()))
        .then_with(|| {
            left.package_identity()
                .bytes()
                .cmp(right.package_identity().bytes())
        })
        .then_with(|| {
            left.schema_identity()
                .bytes()
                .cmp(right.schema_identity().bytes())
        })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationEntityTouchScope {
    schema: ApplicationSchemaBindingIdentity,
    entity: String,
}

impl WorthQueryOperationEntityTouchScope {
    pub(crate) fn new(schema: ApplicationSchemaBindingIdentity, entity: String) -> Self {
        Self { schema, entity }
    }

    pub const fn schema(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }

    pub fn entity(&self) -> &str {
        &self.entity
    }
}

/// Installed field-touch meaning. Only application installation can construct it.
///
/// ```compile_fail
/// use worth_query_installation::facade::WorthQueryOperationFieldTouchScope;
/// let _constructor = WorthQueryOperationFieldTouchScope::new;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationFieldTouchScope {
    schema: ApplicationSchemaBindingIdentity,
    entity: String,
    contract: AspectContract,
    canonical_contract_basis: Arc<CanonicalBasisReadyArtifact>,
    canonical_contract_material: Arc<str>,
    field_path: CanonicalFieldPath,
}

impl WorthQueryOperationFieldTouchScope {
    pub(crate) fn new(
        schema: ApplicationSchemaBindingIdentity,
        entity: String,
        installed: &crate::application_schema::WorthQueryInstalledApplicationAspectContract,
        field_path: CanonicalFieldPath,
    ) -> Self {
        Self {
            schema,
            entity,
            contract: installed.contract().clone(),
            canonical_contract_basis: installed.retain_canonical_contract_basis(),
            canonical_contract_material: installed.retain_canonical_contract_material(),
            field_path,
        }
    }

    pub const fn schema(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }

    pub fn entity(&self) -> &str {
        &self.entity
    }

    pub const fn contract(&self) -> &AspectContract {
        &self.contract
    }

    pub(crate) fn canonical_contract_basis(&self) -> &CanonicalBasisReadyArtifact {
        self.canonical_contract_basis.as_ref()
    }

    pub(crate) fn canonical_contract_material(&self) -> &str {
        self.canonical_contract_material.as_ref()
    }

    pub const fn field_path(&self) -> &CanonicalFieldPath {
        &self.field_path
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationRelationTouchScope {
    schema: ApplicationSchemaBindingIdentity,
    relation: String,
    from: String,
    to: String,
}

impl WorthQueryOperationRelationTouchScope {
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

/// Atomic semantic identity used only by portable domain-operation contracts.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryDeclaredDomainTouchScopeIdentity(String);

impl WorthQueryDeclaredDomainTouchScopeIdentity {
    pub fn new(value: impl Into<String>) -> Result<Self, &'static str> {
        let value = value.into();
        if value.is_empty()
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            return Err("invalid-declared-domain-touch-scope-identity");
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
