use worth_query_decl::facade::{
    application_query::{
        ApplicationLiveQueryIntent, ApplicationQueryBasisSupport, ApplicationQueryCardinality,
        ApplicationQueryDefinition, ApplicationQueryDefinitionBuilder,
        ApplicationQueryDependencyCeiling, ApplicationQueryDisclosureContract,
        ApplicationQueryLaneEligibility, ApplicationQueryLiveResourceContract,
        ApplicationQueryParameterSet, ApplicationQueryResultFieldRef,
        ApplicationQueryResultShapeBuilder,
    },
    application_schema::{
        EqualityPredicate, NoApplicationUnit, ReadOnly, ReadWrite, U64ApplicationValueBinding,
    },
    worth_query_application_query, worth_query_portable_type, worth_query_query_binding,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

use super::{
    IdentityAspect, IdentityIdField, QueryRevisionAspect, QueryRevisionValueField, QueryTextAspect,
    QueryTextStatusField, WorthUiApplicationSchema, WorthUiExternalPrincipalMapping,
    WorthUiPrincipal, WorthUiPrincipalBinding, WorthUiRecord, WorthUiStatusLiveCause,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiStatusQueryResult {
    pub identity: String,
    pub status: String,
    pub revision: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthUiStatusQueryParameters;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiStatusQueryRequest {
    identity: String,
}

impl WorthUiStatusQueryRequest {
    pub fn new(identity: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
        }
    }

    fn identity(self) -> String {
        self.identity
    }
}

worth_query_structured_value_binding!(pub WorthUiStatusQueryParametersBinding for WorthUiStatusQueryParameters {
    identity: "worth.ui.status-query-parameters.v1"
});
worth_query_structured_value_binding!(pub WorthUiStatusQueryResultBinding for WorthUiStatusQueryResult {
    identity: "worth.ui.status-query-result.v1"
});
worth_query_structured_value_binding!(pub WorthUiStatusQueryRequestBinding for WorthUiStatusQueryRequest {
    identity: "worth.ui.status-query-request.v1"
});
worth_query_application_query!(
    pub WorthUiStatusQuery for WorthUiApplicationSchema,
    identity "worth.ui.status-query.v1",
    parameters WorthUiStatusQueryParametersBinding,
    result WorthUiStatusQueryResultBinding,
    scope WorthUiRecord => "WorthUiRecord",
    name "status"
);
worth_query_query_binding!(
    pub WorthUiStatusQueryBinding for WorthUiStatusQueryRequest,
    schema WorthUiApplicationSchema,
    identity "worth.ui.status-query-binding.v1",
    input WorthUiStatusQueryRequestBinding,
    query WorthUiStatusQuery,
    parameters WorthUiStatusQueryParametersBinding => |_| ApplicationQueryParameterSet::new(),
    result WorthUiStatusQueryResultBinding,
    principal WorthUiPrincipalBinding,
    mapping WorthUiExternalPrincipalMapping,
    principal_entity WorthUiPrincipal,
    principal_identity u64,
    identity_binding U64ApplicationValueBinding,
    scope WorthUiRecord, IdentityAspect, IdentityIdField, String, ReadOnly, NoApplicationUnit,
    field IdentityIdField::reference(),
    value WorthUiStatusQueryRequest::identity,
    limits results 1, work 64
);

impl ApplicationLiveQueryIntent<WorthUiApplicationSchema> for WorthUiStatusQueryRequest {
    type Target = WorthUiRecord;
    type LiveCause = WorthUiStatusLiveCause;
}

struct IdentitySlot;
worth_query_portable_type!(IdentitySlot => "worth.ui.status-query.identity.v1");
struct StatusSlot;
worth_query_portable_type!(StatusSlot => "worth.ui.status-query.status.v1");
struct RevisionSlot;
worth_query_portable_type!(RevisionSlot => "worth.ui.status-query.revision.v1");

type IdentitySelector = ApplicationQueryResultFieldRef<
    WorthUiStatusQuery,
    IdentitySlot,
    WorthUiApplicationSchema,
    WorthUiRecord,
    IdentityAspect,
    IdentityIdField,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
>;
type StatusSelector = ApplicationQueryResultFieldRef<
    WorthUiStatusQuery,
    StatusSlot,
    WorthUiApplicationSchema,
    WorthUiRecord,
    QueryTextAspect,
    QueryTextStatusField,
    String,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
>;
type RevisionSelector = ApplicationQueryResultFieldRef<
    WorthUiStatusQuery,
    RevisionSlot,
    WorthUiApplicationSchema,
    WorthUiRecord,
    QueryRevisionAspect,
    QueryRevisionValueField,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
>;

fn identity() -> IdentitySelector {
    ApplicationQueryResultFieldRef::new("identity", IdentityIdField::reference())
}

fn status() -> StatusSelector {
    ApplicationQueryResultFieldRef::new("status", QueryTextStatusField::reference())
}

fn revision() -> RevisionSelector {
    ApplicationQueryResultFieldRef::new("revision", QueryRevisionValueField::reference())
}

pub(crate) fn status_query_definition() -> ApplicationQueryDefinition<
    WorthUiApplicationSchema,
    WorthUiStatusQuery,
    WorthUiStatusQueryParameters,
    WorthUiStatusQueryResult,
    WorthUiRecord,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        WorthUiApplicationSchema,
        WorthUiStatusQuery,
        WorthUiRecord,
        WorthUiStatusQueryResult,
        WorthUiStatusQueryResultBinding,
    >::new(WorthUiRecord::reference())
    .field(identity())
    .field(status())
    .field(revision())
    .build();
    ApplicationQueryDefinitionBuilder::declare(WorthUiStatusQuery::reference())
        .root(WorthUiRecord::reference())
        .scope(WorthUiRecord::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::OptionalOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 3))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_live())
        .public()
        .live_root_by::<WorthUiStatusLiveCause, _, _, _, _, _, _, _, _>(
            identity(),
            identity(),
            ApplicationQueryLiveResourceContract::bounded(4, 512, 1_024),
        )
        .build()
        .expect("the UI status query is statically canonical")
}

impl WorthQueryApplicationProjection<WorthUiApplicationSchema, WorthUiStatusQuery>
    for WorthUiStatusQueryResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, WorthUiApplicationSchema, WorthUiStatusQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            identity: row.field(identity())?,
            status: row.field(status())?,
            revision: row.field(revision())?,
        })
    }
}
