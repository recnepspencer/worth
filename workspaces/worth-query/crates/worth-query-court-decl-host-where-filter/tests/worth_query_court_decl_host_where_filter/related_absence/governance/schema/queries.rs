//! Governed query declarations exercising `use_relation_by` and target-only
//! `use_field_by` (`Gate` is NEVER used on the root, only in the related target
//! predicate!) on `where_no_related` queries.
use super::*;
use worth_query_decl::facade::application_query::*;
use worth_query_decl::facade::application_schema::fragment::Registration;
use worth_query_decl::facade::*;
use worth_query_host::facade::primary_graph::*;

worth_query_decl::facade::worth_query_disclosure_classification!(DenyFldDisclosure);
worth_query_decl::facade::worth_query_disclosure_classification!(DenyRelDisclosure);
worth_query_decl::facade::worth_query_disclosure_classification!(NoRelDisclosure);
worth_query_decl::facade::worth_query_disclosure_classification!(NoTfldDisclosure);
worth_query_decl::facade::worth_query_disclosure_classification!(OkDisclosure);

pub struct GovParams;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovRow {
    pub key: u64,
}
worth_query_structured_value_binding!(pub GovParamsBinding for GovParams { identity: "related-absence.gov.params.v1" });
worth_query_structured_value_binding!(pub GovRowBinding for GovRow { identity: "related-absence.gov.row.v1" });
pub struct KeySlot;
worth_query_portable_type!(KeySlot => "related-absence.gov.key-slot.v1");
pub fn key<Q>() -> ApplicationQueryResultFieldRef<Q, KeySlot, GovSchema, TaskKey> {
    ApplicationQueryResultFieldRef::new("key", TaskKey::at("TaskKey"))
}

fn str_val(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value.to_owned()).unwrap()
}

macro_rules! gov_query {
    ($q:ident, $id:literal, $name:literal, $intent:ident, $scope_fn:ident, $in_bind:ident, $bind:ident, $bind_id:literal) => {
        worth_query_application_query!(pub $q for GovSchema, identity $id,
            parameters GovParamsBinding, result GovRowBinding, scope Project => "Project", name $name);
        impl WorthQueryApplicationProjection<GovSchema, $q> for GovRow {
            fn project(row: &WorthQueryApplicationProjectionRow<'_, GovSchema, $q>) -> Result<Self, WorthQueryApplicationProjectionDenial> {
                Ok(Self { key: row.field(key())? })
            }
        }
        #[derive(Clone, Debug, WorthValue)]
        pub struct $intent { pub project: String }
        fn $scope_fn(i: $intent) -> String { i.project }
        worth_query_structured_value_binding!(pub $in_bind for $intent { identity: $bind_id });
        worth_query_query_binding!(pub $bind for $intent, schema GovSchema, identity $bind_id,
            input $in_bind, query $q, parameters GovParamsBinding => |_: &$intent| ApplicationQueryParameterSet::new(),
            result GovRowBinding, principal GovPrincipalBinding, mapping ExternalMapping,
            principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
            scope ProjectKey, field ProjectKey::at("ProjectKey"), value $scope_fn, limits results 64);
    };
}
gov_query!(
    GovAllowedQuery,
    "related-absence.gov.ok.v1",
    "gov_ok",
    GovAllowedIntent,
    gov_ok_scope,
    GovAllowedIn,
    GovAllowedBind,
    "related-absence.gov.ok-b.v1"
);
gov_query!(
    GovMissingRelRuleQuery,
    "related-absence.gov.no-rule.v1",
    "gov_no_rule",
    GovMissingRelIntent,
    gov_no_rule_scope,
    GovMissingRelIn,
    GovMissingRelBind,
    "related-absence.gov.no-rule-b.v1"
);
gov_query!(
    GovMissingTargetFieldRuleQuery,
    "related-absence.gov.no-tfld.v1",
    "gov_no_tfld",
    GovMissingTargetFieldIntent,
    gov_no_tfld_scope,
    GovMissingTargetFieldIn,
    GovMissingTargetFieldBind,
    "related-absence.gov.no-tfld-b.v1"
);
gov_query!(
    GovDeniedRelQuery,
    "related-absence.gov.deny-rel.v1",
    "gov_deny_rel",
    GovDeniedRelIntent,
    gov_deny_rel_scope,
    GovDeniedRelIn,
    GovDeniedRelBind,
    "related-absence.gov.deny-rel-b.v1"
);
gov_query!(
    GovDeniedFieldQuery,
    "related-absence.gov.deny-fld.v1",
    "gov_deny_fld",
    GovDeniedFieldIntent,
    gov_deny_fld_scope,
    GovDeniedFieldIn,
    GovDeniedFieldBind,
    "related-absence.gov.deny-fld-b.v1"
);

fn gov_def<
    Q: ApplicationQueryMarkerIdentity<GovSchema, ResultBinding = GovRowBinding> + 'static,
>(
    reference: ApplicationQueryReference<GovSchema, Q, GovParams, GovRow, Project>,
    disclosure: ApplicationQueryDisclosureContract,
) -> ApplicationQueryDefinitionBuilder<GovSchema, Q, GovParams, GovRow, Project> {
    // Root uses ONLY `TaskKey` and `Status`. `Gate` is referenced EXCLUSIVELY
    // inside the related-predicate target matcher so target-field governance
    // is tested completely independently of root guards.
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Task::reference())
        .scope(Project::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<GovSchema, Q, Task, GovRow, GovRowBinding>::new(
                Task::reference(),
            )
            .field(key())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(ReadBoardRows::reference())
        .root_path(
            ApplicationQueryRootPath::from(Project::reference()).forward(Contains::reference()),
        )
        .where_equal_value(Status::at("Status"), str_val("open"))
        .where_no_related(
            ApplicationQueryRelatedPredicate::forward(BlockedBy::reference())
                .where_equal_value(Gate::at("Gate"), str_val("blocking")),
        )
        .order_by(key(), ApplicationQueryOrderingDirection::Ascending)
}

pub struct GovQueries;
impl Registration<GovSchema> for GovQueries {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<GovSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<GovSchema>, ApplicationSchemaDeclarationDenial>
    {
        let all = ApplicationQueryInfluenceContract::permit_all();
        let root_disc = |governed: ApplicationQueryDisclosureContract| {
            governed
                .use_field_by(TaskKey::at("TaskKey"), tag(1), all.clone())
                .unwrap()
                .use_field_by(Status::at("Status"), tag(1), all.clone())
                .unwrap()
        };
        let ok_disc = root_disc(ApplicationQueryDisclosureContract::governed_by(
            OkDisclosure,
            ReadBoard::reference(),
        ))
        .use_relation_by(
            BlockedBy::reference(),
            RelationDirection::Forward,
            tag(1),
            all.clone(),
        )
        .use_field_by(Gate::at("Gate"), tag(1), all.clone())
        .unwrap()
        .disclose_field_by(key::<GovAllowedQuery>(), tag(1), all.clone());
        let no_rel_disc = root_disc(ApplicationQueryDisclosureContract::governed_by(
            NoRelDisclosure,
            ReadBoard::reference(),
        ))
        .use_field_by(Gate::at("Gate"), tag(1), all.clone())
        .unwrap()
        .disclose_field_by(key::<GovMissingRelRuleQuery>(), tag(1), all.clone());
        let no_tfld_disc = root_disc(ApplicationQueryDisclosureContract::governed_by(
            NoTfldDisclosure,
            ReadBoard::reference(),
        ))
        .use_relation_by(
            BlockedBy::reference(),
            RelationDirection::Forward,
            tag(1),
            all.clone(),
        )
        .disclose_field_by(key::<GovMissingTargetFieldRuleQuery>(), tag(1), all.clone());
        let deny_rel_disc = root_disc(ApplicationQueryDisclosureContract::governed_by(
            DenyRelDisclosure,
            ReadBoard::reference(),
        ))
        .use_relation_by(
            BlockedBy::reference(),
            RelationDirection::Forward,
            tag(2),
            all.clone(),
        )
        .use_field_by(Gate::at("Gate"), tag(1), all.clone())
        .unwrap()
        .disclose_field_by(key::<GovDeniedRelQuery>(), tag(1), all.clone());
        // Root fields (`TaskKey`, `Status`) and `BlockedBy` relation are ALL tag 1 (authorized).
        // ONLY target-only field `Gate` requires tag 2 (unauthorized).
        let deny_fld_disc = root_disc(ApplicationQueryDisclosureContract::governed_by(
            DenyFldDisclosure,
            ReadBoard::reference(),
        ))
        .use_relation_by(
            BlockedBy::reference(),
            RelationDirection::Forward,
            tag(1),
            all.clone(),
        )
        .use_field_by(Gate::at("Gate"), tag(2), all.clone())
        .unwrap()
        .disclose_field_by(key::<GovDeniedFieldQuery>(), tag(1), all);
        Ok(builder
            .application_query(gov_def(GovAllowedQuery::reference(), ok_disc))
            .application_query_binding::<GovAllowedBind>()
            .application_query(gov_def(GovMissingRelRuleQuery::reference(), no_rel_disc))
            .application_query_binding::<GovMissingRelBind>()
            .application_query(gov_def(
                GovMissingTargetFieldRuleQuery::reference(),
                no_tfld_disc,
            ))
            .application_query_binding::<GovMissingTargetFieldBind>()
            .application_query(gov_def(GovDeniedRelQuery::reference(), deny_rel_disc))
            .application_query_binding::<GovDeniedRelBind>()
            .application_query(gov_def(GovDeniedFieldQuery::reference(), deny_fld_disc))
            .application_query_binding::<GovDeniedFieldBind>())
    }
}
