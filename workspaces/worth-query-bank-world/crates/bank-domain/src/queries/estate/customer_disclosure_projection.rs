use worth_query_decl::facade::application_schema::ApplicationReadableScalarValueBinding;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationDisclosed, WorthQueryApplicationOmission, WorthQueryApplicationProjection,
    WorthQueryApplicationProjectionDenial, WorthQueryApplicationProjectionRow,
};

use crate::estate::{BankDisclosure, RestrictedBankField};
use crate::schema::{BankSchema, RestrictedBankFieldBinding};

use super::customer_disclosure::{EstateCustomerDisclosure, EstateCustomerDisclosureQuery};
use super::customer_disclosure_selectors::{
    beneficiary_identity, customer_identity, estate_beneficiaries, estate_customer,
};

impl WorthQueryApplicationProjection<BankSchema, EstateCustomerDisclosureQuery>
    for EstateCustomerDisclosure
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, BankSchema, EstateCustomerDisclosureQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let customer = match row.disclosed_one(estate_customer())? {
            WorthQueryApplicationDisclosed::Disclosed(customer) => {
                match customer.disclosed_field(customer_identity())? {
                    WorthQueryApplicationDisclosed::Disclosed(identity) => {
                        BankDisclosure::Disclosed(identity)
                    }
                    WorthQueryApplicationDisclosed::Omitted(omission) => {
                        omission_value(omission, RestrictedBankField::CustomerIdentity)?
                    }
                }
            }
            WorthQueryApplicationDisclosed::Omitted(omission) => {
                omission_value(omission, RestrictedBankField::CustomerIdentity)?
            }
        };
        let beneficiaries = match row.disclosed_many(estate_beneficiaries())? {
            WorthQueryApplicationDisclosed::Disclosed(rows) => {
                let identities = rows
                    .iter()
                    .map(|beneficiary| beneficiary.disclosed_field(beneficiary_identity()))
                    .collect::<Result<Vec<_>, _>>()?;
                let mut disclosed = Vec::with_capacity(identities.len());
                for identity in identities {
                    match identity {
                        WorthQueryApplicationDisclosed::Disclosed(identity) => {
                            disclosed.push(identity);
                        }
                        WorthQueryApplicationDisclosed::Omitted(omission) => {
                            return Ok(EstateCustomerDisclosure::new(
                                customer,
                                omission_value(omission, RestrictedBankField::BeneficiaryIdentity)?,
                            ));
                        }
                    }
                }
                BankDisclosure::Disclosed(disclosed)
            }
            WorthQueryApplicationDisclosed::Omitted(omission) => {
                omission_value(omission, RestrictedBankField::BeneficiaryIdentity)?
            }
        };
        Ok(EstateCustomerDisclosure::new(customer, beneficiaries))
    }
}

fn omission_value<T>(
    omission: WorthQueryApplicationOmission,
    expected: RestrictedBankField,
) -> Result<BankDisclosure<T>, WorthQueryApplicationProjectionDenial> {
    let field = RestrictedBankFieldBinding::decode(omission.required_disclosure())
        .ok()
        .filter(|field| *field == expected)
        .ok_or_else(|| WorthQueryApplicationProjectionDenial::reject("customer-disclosure"))?;
    Ok(BankDisclosure::Omitted(field.classification()))
}
