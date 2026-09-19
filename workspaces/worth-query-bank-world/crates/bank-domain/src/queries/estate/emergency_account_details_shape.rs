use worth_query_decl::facade::application_query::{
    ApplicationQueryResultShapeBuilder, TypedApplicationQueryResultShape,
};

use crate::schema::{Account, BankSchema, EstateCase};

use super::emergency_account_details::{
    EstateEmergencyAccountDetails, EstateEmergencyAccountDetailsQuery,
};
use super::emergency_account_details_selectors::{
    account_identity, account_name, account_status, estate_account,
};

pub(super) fn emergency_account_details_shape() -> TypedApplicationQueryResultShape<
    BankSchema,
    EstateEmergencyAccountDetailsQuery,
    EstateCase,
    EstateEmergencyAccountDetails,
    super::emergency_account_details::EstateEmergencyAccountDetailsQueryResultBinding,
> {
    let account = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        EstateEmergencyAccountDetailsQuery,
        Account,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Account::reference())
    .field(account_identity())
    .field(account_name())
    .field(account_status());
    ApplicationQueryResultShapeBuilder::new(EstateCase::reference())
        .relation(estate_account(), account)
        .build()
}
