//! Top-down declarations own membership; members never repeat their parent.

use super::super::{
    ApplicationRelationIntegrity, ApplicationSchema, ApplicationSchemaDeclaration,
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaDeclarationDenial,
};
use super::lowering::{FragmentMembers, Lowering};

pub trait Fragment: Sized + 'static {
    const NAME: &'static str;
    type Bindings: super::BindingSet;
    type Members;
}

pub trait Entity: 'static {
    const NAME: &'static str;
    type Aspects;
    type Relations;
}

impl<S, E: Entity> super::super::ApplicationEntityMarkerIdentity<S> for E {
    const IDENTIFIER: &'static str = E::NAME;
}

pub trait Aspect: 'static {
    const NAME: &'static str;
    type Row: 'static;
}

pub trait Relation: 'static {
    const NAME: &'static str;
    type To: Entity;
    const SELF_EDGES: bool = true;
    const CARDINALITY: super::super::ApplicationRelationCardinality =
        super::super::ApplicationRelationCardinality::unbounded();
    const DELETION: super::super::ApplicationRelationDeletionPolicy =
        super::super::ApplicationRelationDeletionPolicy::RetainDanglingForAudit;
    const CROSS_CONTEXT: super::super::ApplicationRelationCrossContextPolicy =
        super::super::ApplicationRelationCrossContextPolicy::Forbid;
    const ACYCLIC: bool = false;
    const SYMMETRIC: bool = false;
    const ORBIT: bool = false;
    const UNIQUE: Option<super::super::ApplicationRelationUniquenessScope> = None;
    const SAME_PARTITION_ENDPOINTS: bool = false;
    const REACHES_AT_LEAST: Option<super::super::ApplicationRelationMinimumReachability> = None;
}

/// Derives descriptive integrity from the relation declaration.
pub const fn relation_integrity<R: Relation>() -> ApplicationRelationIntegrity {
    ApplicationRelationIntegrity {
        endpoints: super::super::ApplicationRelationEndpoints::new(R::SELF_EDGES, R::CROSS_CONTEXT),
        cardinality: R::CARDINALITY,
        deletion: R::DELETION,
        acyclic: R::ACYCLIC,
        symmetric: R::SYMMETRIC,
        orbit: R::ORBIT,
        unique: R::UNIQUE,
        same_partition_endpoints: R::SAME_PARTITION_ENDPOINTS,
        reaches_at_least: R::REACHES_AT_LEAST,
    }
}

/// Operation/query/capability registrations retain their existing definition
/// builders. Listing the marker in Members owns its application membership.
pub trait Registration<Schema>: 'static {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>;
}

/// Materializes a named installation root. The traversal witness is inferred
/// within this call and never enters the declaration's public type.
pub fn declare<F, Witness>(
    fragment: F,
) -> Result<ApplicationSchemaDeclaration<F>, ApplicationSchemaDeclarationDenial>
where
    F: Fragment + ApplicationSchema,
    F::Members: FragmentMembers<F, F::Bindings, Witness>,
{
    declaration_builder(fragment).and_then(|builder| builder.build())
}
/// Lowers tuple membership before authoring additional operation/query contracts.
/// Model membership is always supplied by `F::Members`.
pub fn declaration_builder<F, Witness>(
    fragment: F,
) -> Result<ApplicationSchemaDeclarationBuilder<F>, ApplicationSchemaDeclarationDenial>
where
    F: Fragment + ApplicationSchema,
    F::Members: FragmentMembers<F, F::Bindings, Witness>,
{
    ApplicationSchemaDeclarationBuilder::from_fragment(fragment)
}

impl<Schema: ApplicationSchema> ApplicationSchemaDeclarationBuilder<Schema> {
    /// Compiles a fragment's declared membership into this installation root.
    pub fn from_fragment<F, Witness>(_: F) -> Result<Self, ApplicationSchemaDeclarationDenial>
    where
        F: Fragment,
        F::Members: FragmentMembers<Schema, F::Bindings, Witness>,
    {
        let prefix = F::NAME;
        let mut lowering = Lowering::root(prefix)?;
        let mut builder = Self::for_schema();
        builder
            .member_provenance
            .fragments
            .catalog
            .register::<F>(prefix)?;
        F::Members::lower(builder, prefix, &mut lowering)
    }
}

/// Binding-free composition roots contain reusable fragment and registration members.
pub struct NoBindings;
impl super::BindingSet for NoBindings {
    type Work<'a> = crate::application_schema::ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut crate::application_schema::ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
/// Correction dispatch follows the same membership tuple as declaration lowering.
pub fn visit_correction<F, Witness, Visitor>(
    _: F,
    source: std::any::TypeId,
    visitor: Visitor,
) -> Visitor::Output
where
    F: Fragment + ApplicationSchema,
    F::Members: FragmentMembers<F, F::Bindings, Witness>,
    Visitor: crate::application_aftermath::ApplicationCorrectionVisitor<F>,
{
    match F::Members::visit_correction(source, visitor) {
        Ok(output) => output,
        Err(visitor) => visitor.unavailable(),
    }
}
