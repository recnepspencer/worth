use worth_query_decl::facade::application_query::{
    ApplicationQueryResultShapeBuilder, TypedApplicationQueryResultShape,
};

use crate::schema::{BankSchema, EstateCase, Principal};

use super::customer_disclosure::{EstateCustomerDisclosure, EstateCustomerDisclosureQuery};
use super::customer_disclosure_selectors::{
    beneficiary_identity, customer_identity, estate_beneficiaries, estate_customer,
};

pub(super) fn customer_disclosure_shape() -> TypedApplicationQueryResultShape<
    BankSchema,
    EstateCustomerDisclosureQuery,
    EstateCase,
    EstateCustomerDisclosure,
    super::customer_disclosure::EstateCustomerDisclosureQueryResultBinding,
> {
    let customer = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        EstateCustomerDisclosureQuery,
        Principal,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Principal::reference())
    .field(customer_identity());
    let beneficiary = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        EstateCustomerDisclosureQuery,
        Principal,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Principal::reference())
    .field(beneficiary_identity());
    ApplicationQueryResultShapeBuilder::new(EstateCase::reference())
        .relation(estate_customer(), customer)
        .relation(estate_beneficiaries(), beneficiary)
        .build()
}
