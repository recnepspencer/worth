use worth_query_decl::facade::application_schema::ApplicationReadableScalarValueBinding;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationDisclosed, WorthQueryApplicationOmission, WorthQueryApplicationProjection,
    WorthQueryApplicationProjectionDenial, WorthQueryApplicationProjectionRow,
};

use crate::{
    estate::{BankDisclosure, RestrictedBankField},
    reads::EstateAccountView,
    schema::{BankSchema, RestrictedBankFieldBinding},
};

use super::emergency_account_details::{
    EstateEmergencyAccountDetails, EstateEmergencyAccountDetailsQuery,
};
use super::emergency_account_details_selectors::{
    account_identity, account_name, account_status, estate_account,
};

impl WorthQueryApplicationProjection<BankSchema, EstateEmergencyAccountDetailsQuery>
    for EstateEmergencyAccountDetails
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<
            '_,
            BankSchema,
            EstateEmergencyAccountDetailsQuery,
        >,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self::new(project_account(row)?))
    }
}

fn project_account(
    row: &WorthQueryApplicationProjectionRow<'_, BankSchema, EstateEmergencyAccountDetailsQuery>,
) -> Result<BankDisclosure<EstateAccountView>, WorthQueryApplicationProjectionDenial> {
    let account = match row.disclosed_one(estate_account())? {
        WorthQueryApplicationDisclosed::Disclosed(account) => account,
        WorthQueryApplicationDisclosed::Omitted(omission) => return omission_value(omission),
    };
    let identity = match account.disclosed_field(account_identity())? {
        WorthQueryApplicationDisclosed::Disclosed(identity) => identity,
        WorthQueryApplicationDisclosed::Omitted(omission) => return omission_value(omission),
    };
    let name = match account.disclosed_field(account_name())? {
        WorthQueryApplicationDisclosed::Disclosed(name) => name,
        WorthQueryApplicationDisclosed::Omitted(omission) => return omission_value(omission),
    };
    let status = match account.disclosed_field(account_status())? {
        WorthQueryApplicationDisclosed::Disclosed(status) => status,
        WorthQueryApplicationDisclosed::Omitted(omission) => return omission_value(omission),
    };
    Ok(BankDisclosure::Disclosed(
        EstateAccountView::from_projection(identity, name, status),
    ))
}

fn omission_value<T>(
    omission: WorthQueryApplicationOmission,
) -> Result<BankDisclosure<T>, WorthQueryApplicationProjectionDenial> {
    let field = RestrictedBankFieldBinding::decode(omission.required_disclosure())
        .ok()
        .filter(|field| *field == RestrictedBankField::AccountDetails)
        .ok_or_else(|| {
            WorthQueryApplicationProjectionDenial::reject("emergency-account-details")
        })?;
    Ok(BankDisclosure::Omitted(field.classification()))
}
