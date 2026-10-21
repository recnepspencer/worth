//! Real root/child membership, with no canonical value decoded during its census.
use super::super::{query, schema::*};
use worth_query_host::facade::{
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};
use worth_relational::facade::identity::EntityId;

pub struct Members {
    pub rows: Vec<(EntityId, String)>,
}
pub struct IdentitySlot;
pub struct ChildIdentitySlot;
pub struct ChildrenSlot;
worth_query_portable_type!(IdentitySlot => "canonical-admission.paired.identity-slot.v1");
worth_query_portable_type!(ChildIdentitySlot => "canonical-admission.paired.child-identity-slot.v1");
worth_query_portable_type!(ChildrenSlot => "canonical-admission.paired.children-slot.v1");
worth_query_structured_value_binding!(pub MembersBinding for Members { identity: "canonical-admission.paired.members.v1" });
worth_query_structured_value_binding!(pub ChildBinding for () { identity: "canonical-admission.paired.member-child.v1" });
worth_query_application_query!(pub MembershipQuery for AdmissionSchema,
    identity "canonical-admission.paired.membership.v1", parameters query::ParametersBinding,
    result MembersBinding, scope Document => "Document", name "paired_canonical_membership");

fn identity() -> ApplicationQueryResultFieldRef<
    MembershipQuery,
    IdentitySlot,
    AdmissionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("identity", DocumentIdentityField::reference())
}

fn children() -> ApplicationQueryResultRelationRef<
    MembershipQuery,
    ChildrenSlot,
    AdmissionSchema,
    ChildLink,
    Document,
    Document,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("children", ChildLink::reference())
}

fn child_identity() -> ApplicationQueryResultFieldRef<
    MembershipQuery,
    ChildIdentitySlot,
    AdmissionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("child_identity", DocumentIdentityField::reference())
}

pub fn definition() -> ApplicationQueryDefinition<
    AdmissionSchema,
    MembershipQuery,
    query::Parameters,
    Members,
    Document,
> {
    let child = ApplicationQueryResultShapeBuilder::<
        AdmissionSchema,
        MembershipQuery,
        Document,
        (),
        ChildBinding,
    >::new(Document::reference())
    .field(child_identity());
    let shape = ApplicationQueryResultShapeBuilder::<
        AdmissionSchema,
        MembershipQuery,
        Document,
        Members,
        MembersBinding,
    >::new(Document::reference())
    .field(identity())
    .relation(children(), child)
    .build();
    ApplicationQueryDefinitionBuilder::declare(MembershipQuery::reference())
        .root(Document::reference())
        .scope(Document::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 4))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<AdmissionSchema, MembershipQuery> for Members {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, AdmissionSchema, MembershipQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let mut rows = vec![(row.entity_id(), row.field(identity())?)];
        for child in row.many(children())?.iter() {
            rows.push((child.entity_id(), child.field(child_identity())?));
        }
        Ok(Self { rows })
    }
}

pub struct MembershipBinding;
type ReadScope = ApplicationQueryFieldScope<
    AdmissionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationQueryBinding<AdmissionSchema> for MembershipBinding {
    type Input = query::ReadValue;
    type InputBinding = query::ReadInputBinding;
    type Query = MembershipQuery;
    type ParameterBinding = query::ParametersBinding;
    type ResultBinding = MembersBinding;
    type ScopeBinding = ReadScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "canonical-admission.paired.membership-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        AdmissionSchema,
        Document,
        DocumentFacts,
        DocumentIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        DocumentIdentityField::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        AdmissionSchema,
        DocumentPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DocumentPrincipalBinding::reference()
    }
}
