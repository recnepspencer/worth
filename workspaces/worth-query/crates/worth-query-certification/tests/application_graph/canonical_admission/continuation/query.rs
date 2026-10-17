//! Ordered native child pages keep admitted nested values through finalization.
use super::super::{model::NestedValue, opaque::SwallowingValue, query, schema::*};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryCancellationSource,
    declaration::{application_query::*, application_schema::*},
    primary_graph::*,
    *,
};

#[derive(Debug, PartialEq)]
pub struct PageRow {
    pub children: Vec<(String, NestedValue)>,
}
pub struct ChildIdentitySlot;
pub struct ChildValueSlot;
pub struct ChildrenSlot;
worth_query_portable_type!(ChildIdentitySlot => "canonical-admission.continuation.child-identity-slot.v1");
worth_query_portable_type!(ChildValueSlot => "canonical-admission.continuation.child-value-slot.v1");
worth_query_portable_type!(ChildrenSlot => "canonical-admission.continuation.children-slot.v1");
worth_query_structured_value_binding!(pub PageResultBinding for PageRow { identity: "canonical-admission.continuation.result.v1" });
worth_query_structured_value_binding!(pub ChildBinding for () { identity: "canonical-admission.continuation.child.v1" });
worth_query_application_query!(pub PageQuery for AdmissionSchema,
    identity "canonical-admission.continuation.query.v1", parameters query::ParametersBinding,
    result PageResultBinding, scope Document => "Document", name "canonical_child_page");

fn child_identity() -> ApplicationQueryResultFieldRef<
    PageQuery,
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
    ApplicationQueryResultFieldRef::new("identity", DocumentIdentityField::reference())
}
fn child_value() -> ApplicationQueryResultFieldRef<
    PageQuery,
    ChildValueSlot,
    AdmissionSchema,
    Document,
    DocumentFacts,
    DocumentValueField,
    SwallowingValue,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("value", DocumentValueField::reference())
}
fn children() -> ApplicationQueryResultRelationRef<
    PageQuery,
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

pub fn definition(
) -> ApplicationQueryDefinition<AdmissionSchema, PageQuery, query::Parameters, PageRow, Document> {
    let child = ApplicationQueryResultShapeBuilder::<
        AdmissionSchema,
        PageQuery,
        Document,
        (),
        ChildBinding,
    >::new(Document::reference())
    .field(child_identity())
    .field(child_value());
    let shape = ApplicationQueryResultShapeBuilder::<
        AdmissionSchema,
        PageQuery,
        Document,
        PageRow,
        PageResultBinding,
    >::new(Document::reference())
    .relation(children(), child)
    .build();
    ApplicationQueryDefinitionBuilder::declare(PageQuery::reference())
        .root(Document::reference())
        .scope(Document::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 4))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .order_by(
            child_identity(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .continue_by(children())
        .build()
        .unwrap()
}
impl WorthQueryApplicationProjection<AdmissionSchema, PageQuery> for PageRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, AdmissionSchema, PageQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let mut children_result = Vec::new();
        for child in row.many(children())?.iter() {
            let identity = child.field(child_identity())?;
            let value = child.field(child_value())?.0;
            // Both opaque seed and projector catch the genuine second refusal.
            // Native completion must reject their apparent success.
            let _ = child.field(child_value());
            children_result.push((identity, value));
        }
        RETURNED.set(RETURNED.get() + 1);
        CANCEL_AFTER_PROJECTION.with(|source| {
            if let Some(source) = source.borrow_mut().take() {
                source.cancel();
            }
        });
        if REJECT_AFTER_PROJECTION.get() {
            return Err(WorthQueryApplicationProjectionDenial::reject(
                "continuation-domain-rejection",
            ));
        }
        Ok(Self {
            children: children_result,
        })
    }
}

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadPage {
    pub identity: String,
}
worth_query_structured_value_binding!(pub PageInputBinding for ReadPage { identity: "canonical-admission.continuation.input.v1" });
pub struct PageBinding;
type ReadScope = ApplicationQueryFieldScope<
    AdmissionSchema,
    Document,
    DocumentFacts,
    DocumentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
impl ApplicationQueryBinding<AdmissionSchema> for PageBinding {
    type Input = ReadPage;
    type InputBinding = PageInputBinding;
    type Query = PageQuery;
    type ParameterBinding = query::ParametersBinding;
    type ResultBinding = PageResultBinding;
    type ScopeBinding = ReadScope;
    type PrincipalBinding = DocumentPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "canonical-admission.continuation.binding.v1";
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
impl ApplicationQueryIntent<AdmissionSchema> for ReadPage {
    type Binding = PageBinding;
    fn parameters(&self) -> ApplicationQueryParameterSet<PageQuery> {
        ApplicationQueryParameterSet::new()
    }
    fn into_scope(self) -> ReadScope {
        ReadScope::new(DocumentIdentityField::reference(), self.identity)
    }
}

thread_local! {
    static RETURNED: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    static CANCEL_AFTER_PROJECTION: std::cell::RefCell<Option<WorthQueryCancellationSource>> = const { std::cell::RefCell::new(None) };
    static REJECT_AFTER_PROJECTION: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
pub fn reset(cancellation: Option<WorthQueryCancellationSource>, reject: bool) {
    RETURNED.set(0);
    REJECT_AFTER_PROJECTION.set(reject);
    CANCEL_AFTER_PROJECTION.with(|source| *source.borrow_mut() = cancellation);
}
pub fn returned() -> u32 {
    RETURNED.get()
}
