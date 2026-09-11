use worth_query_decl::facade::application_query::{
    ApplicationQueryResultShapeBuilder, TypedApplicationQueryResultShape,
};

use crate::reads::EstateCaseOverview;
use crate::schema::{
    Account, BankSchema, Branch, DeathNotice, EmployeeAssignment, EstateCase, LegalAuthority,
    MandatoryReview, Principal,
};

use super::overview::EstateCaseOverviewQuery;
use super::overview_fields::*;
use super::overview_relations::*;

pub(super) fn overview_shape() -> TypedApplicationQueryResultShape<
    BankSchema,
    EstateCaseOverviewQuery,
    EstateCase,
    EstateCaseOverview,
    super::overview::EstateCaseOverviewQueryResultBinding,
> {
    let account = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        EstateCaseOverviewQuery,
        Account,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Account::reference())
    .field(account_identity())
    .field(account_name())
    .field(account_status());
    let branch = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        EstateCaseOverviewQuery,
        Branch,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Branch::reference())
    .field(branch_identity());
    let notice = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        EstateCaseOverviewQuery,
        DeathNotice,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(DeathNotice::reference())
    .field(notice_identity())
    .field(notice_status());
    let deceased = principal_shape(deceased_identity());
    let executor = principal_shape(executor_identity());
    let beneficiary = principal_shape(beneficiary_identity());
    let assignment_principal_shape = principal_shape(assignment_principal_identity());
    let authority_holder_shape = principal_shape(authority_holder_identity());
    let review_reviewer = principal_shape(review_principal_identity());
    let assignment = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        EstateCaseOverviewQuery,
        EmployeeAssignment,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(EmployeeAssignment::reference())
    .field(assignment_identity())
    .field(assignment_role())
    .relation(assignment_principal(), assignment_principal_shape);
    let authority = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        EstateCaseOverviewQuery,
        LegalAuthority,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(LegalAuthority::reference())
    .field(authority_identity())
    .field(authority_kind())
    .field(authority_recognized())
    .relation(authority_holder(), authority_holder_shape);
    let review = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        EstateCaseOverviewQuery,
        MandatoryReview,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(MandatoryReview::reference())
    .field(review_identity())
    .field(review_kind())
    .field(review_status())
    .relation(review_principal(), review_reviewer);

    ApplicationQueryResultShapeBuilder::<
        BankSchema,
        EstateCaseOverviewQuery,
        EstateCase,
        EstateCaseOverview,
        super::overview::EstateCaseOverviewQueryResultBinding,
    >::new(EstateCase::reference())
    .field(estate_identity())
    .field(estate_stage())
    .field(estate_status())
    .relation(estate_account(), account)
    .relation(estate_branch(), branch)
    .relation(estate_notice(), notice)
    .relation(estate_deceased(), deceased)
    .relation(estate_executors(), executor)
    .relation(estate_beneficiaries(), beneficiary)
    .relation(estate_assignments(), assignment)
    .relation(estate_authorities(), authority)
    .relation(estate_reviews(), review)
    .build()
}

fn principal_shape<Slot: worth_query_decl::facade::portable_identity::WorthQueryPortableType>(
    selector: worth_query_decl::facade::application_query::ApplicationQueryResultFieldRef<
        EstateCaseOverviewQuery,
        Slot,
        BankSchema,
        Principal,
        crate::schema::PrincipalIdentity,
        crate::schema::PrincipalIdentityField,
        crate::model::BankPrincipalId,
        worth_query_decl::facade::application_schema::ReadOnly,
        worth_query_decl::facade::application_schema::EqualityPredicate,
        worth_query_decl::facade::application_schema::NoApplicationUnit,
    >,
) -> ApplicationQueryResultShapeBuilder<
    BankSchema,
    EstateCaseOverviewQuery,
    Principal,
    (),
    crate::queries::UnitQueryResultBinding,
> {
    ApplicationQueryResultShapeBuilder::new(Principal::reference()).field(selector)
}
