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

pub(super) const fn relation_integrity<R: Relation>() -> ApplicationRelationIntegrity {
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
    _: F,
) -> Result<ApplicationSchemaDeclaration<F>, ApplicationSchemaDeclarationDenial>
where
    F: Fragment + ApplicationSchema,
    F::Members: FragmentMembers<F, F::Bindings, Witness>,
{
    let prefix = <F as Fragment>::NAME;
    let mut lowering = Lowering::root(prefix)?;
    let mut builder = ApplicationSchemaDeclarationBuilder::for_schema();
    builder
        .member_provenance
        .fragments
        .catalog
        .register::<F>(prefix)?;
    let builder = F::Members::lower(builder, prefix, &mut lowering)?;
    builder.build()
}
