use worth_query_decl::facade::worth_query_application_query;
use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
        ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
        ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef,
        ApplicationQueryResultShapeBuilder, OptionalOneResult, ReverseResultTraversal,
    },
    application_schema::{EqualityPredicate, NoApplicationUnit, ReadOnly},
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

use crate::authorization::ViewAccount;
use crate::model::{AccountId, BankPrincipalId, BusinessId, InstitutionId};
use crate::reads::AccountDetail;
use crate::schema::{
    Account, AccountKind, BankSchema, Business, BusinessAccount, BusinessIdentity,
    BusinessIdentityField, Institution, InstitutionAccount, InstitutionIdentity,
    InstitutionIdentityField, PersonalOwner, Principal, PrincipalIdentity, PrincipalIdentityField,
};

use super::account_summary_projection::{account_summary_shape, project_account_summary};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountDetailQueryParameters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountDetailRequest {
    account: AccountId,
}

impl AccountDetailRequest {
    pub const fn new(account: AccountId) -> Self {
        Self { account }
    }

    pub const fn account(self) -> AccountId {
        self.account
    }
}

pub const fn account_detail(account: AccountId) -> AccountDetailRequest {
    AccountDetailRequest::new(account)
}

worth_query_decl::facade::worth_query_structured_value_binding!(pub AccountDetailQueryParametersBinding for AccountDetailQueryParameters { identity: "AccountDetailQueryParameters" });
worth_query_decl::facade::worth_query_structured_value_binding!(pub AccountDetailQueryResultBinding for AccountDetail { identity: "AccountDetail" });
worth_query_application_query!(
    pub AccountDetailQuery for BankSchema,
    identity "AccountDetailQuery",
    parameters AccountDetailQueryParametersBinding,
    result AccountDetailQueryResultBinding,
    scope Account => "Account",
    name "account_detail"
);

struct InstitutionSlot;
worth_query_decl::facade::worth_query_portable_type!(InstitutionSlot => "InstitutionSlot");
struct InstitutionIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(InstitutionIdentitySlot => "InstitutionIdentitySlot");
struct PersonalOwnerSlot;
worth_query_decl::facade::worth_query_portable_type!(PersonalOwnerSlot => "PersonalOwnerSlot");
struct PrincipalIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(PrincipalIdentitySlot => "PrincipalIdentitySlot");
struct BusinessOwnerSlot;
worth_query_decl::facade::worth_query_portable_type!(BusinessOwnerSlot => "BusinessOwnerSlot");
struct BusinessIdentitySlot;
worth_query_decl::facade::worth_query_portable_type!(BusinessIdentitySlot => "BusinessIdentitySlot");

type InstitutionIdentitySelector = ApplicationQueryResultFieldRef<
    AccountDetailQuery,
    InstitutionIdentitySlot,
    BankSchema,
    Institution,
    InstitutionIdentity,
    InstitutionIdentityField,
    InstitutionId,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
>;

type PrincipalIdentitySelector = ApplicationQueryResultFieldRef<
    AccountDetailQuery,
    PrincipalIdentitySlot,
    BankSchema,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityField,
    BankPrincipalId,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
>;

type BusinessIdentitySelector = ApplicationQueryResultFieldRef<
    AccountDetailQuery,
    BusinessIdentitySlot,
    BankSchema,
    Business,
    BusinessIdentity,
    BusinessIdentityField,
    BusinessId,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
>;

pub fn account_detail_definition() -> ApplicationQueryDefinition<
    BankSchema,
    AccountDetailQuery,
    AccountDetailQueryParameters,
    AccountDetail,
    Account,
> {
    let institution = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        AccountDetailQuery,
        Institution,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Institution::reference())
    .field(institution_identity());
    let personal = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        AccountDetailQuery,
        Principal,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Principal::reference())
    .field(principal_identity());
    let business = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        AccountDetailQuery,
        Business,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Business::reference())
    .field(business_identity());
    let shape = account_summary_shape::<
        AccountDetailQuery,
        AccountDetail,
        AccountDetailQueryResultBinding,
    >()
    .relation(account_institution(), institution)
    .relation(account_personal_owner(), personal)
    .relation(account_business_owner(), business)
    .build();
    ApplicationQueryDefinitionBuilder::declare(AccountDetailQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 4, 9))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(ViewAccount::reference())
        .build()
        .expect("bank account detail query is statically canonical")
}

impl WorthQueryApplicationProjection<BankSchema, AccountDetailQuery> for AccountDetail {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, BankSchema, AccountDetailQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let summary = project_account_summary(row)?;
        let institution = row
            .one(account_institution())?
            .field(institution_identity())?;
        let personal_owner = row
            .optional(account_personal_owner())?
            .map(|owner| owner.field(principal_identity()))
            .transpose()?;
        let business_owner = row
            .optional(account_business_owner())?
            .map(|owner| owner.field(business_identity()))
            .transpose()?;
        match summary.kind() {
            AccountKind::Personal if personal_owner.is_some() && business_owner.is_none() => {}
            AccountKind::Business if personal_owner.is_none() && business_owner.is_some() => {}
            AccountKind::InstitutionCash | AccountKind::InstitutionSettlement
                if personal_owner.is_none() && business_owner.is_none() => {}
            _ => {
                return Err(WorthQueryApplicationProjectionDenial::reject(
                    "account ownership disagrees with account kind",
                ));
            }
        }
        Ok(AccountDetail::from_projection(
            summary,
            institution,
            personal_owner,
            business_owner,
        ))
    }
}

fn institution_identity() -> InstitutionIdentitySelector {
    ApplicationQueryResultFieldRef::new("institution", InstitutionIdentityField::reference())
}

fn principal_identity() -> PrincipalIdentitySelector {
    ApplicationQueryResultFieldRef::new("principal", PrincipalIdentityField::reference())
}

fn business_identity() -> BusinessIdentitySelector {
    ApplicationQueryResultFieldRef::new("business", BusinessIdentityField::reference())
}

fn account_institution() -> ApplicationQueryResultRelationRef<
    AccountDetailQuery,
    InstitutionSlot,
    BankSchema,
    InstitutionAccount,
    Institution,
    Account,
    ReverseResultTraversal,
    worth_query_decl::facade::application_query::ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::reverse_one("institution", InstitutionAccount::reference())
}

fn account_personal_owner() -> ApplicationQueryResultRelationRef<
    AccountDetailQuery,
    PersonalOwnerSlot,
    BankSchema,
    PersonalOwner,
    Principal,
    Account,
    ReverseResultTraversal,
    OptionalOneResult,
> {
    ApplicationQueryResultRelationRef::reverse_optional(
        "personal_owner",
        PersonalOwner::reference(),
    )
}

fn account_business_owner() -> ApplicationQueryResultRelationRef<
    AccountDetailQuery,
    BusinessOwnerSlot,
    BankSchema,
    BusinessAccount,
    Business,
    Account,
    ReverseResultTraversal,
    OptionalOneResult,
> {
    ApplicationQueryResultRelationRef::reverse_optional(
        "business_owner",
        BusinessAccount::reference(),
    )
}
