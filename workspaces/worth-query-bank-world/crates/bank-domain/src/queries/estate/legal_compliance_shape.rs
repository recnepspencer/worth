use worth_query_decl::facade::application_query::{
    ApplicationQueryResultShapeBuilder, TypedApplicationQueryResultShape,
};

use crate::schema::{BankSchema, EstateCase, LegalAuthority, Principal};

use super::{
    legal_compliance::EstateLegalComplianceQuery,
    legal_compliance_projection::EstateLegalComplianceResult, legal_compliance_selectors::*,
};

pub(super) fn legal_compliance_shape() -> TypedApplicationQueryResultShape<
    BankSchema,
    EstateLegalComplianceQuery,
    EstateCase,
    EstateLegalComplianceResult,
    super::legal_compliance::EstateLegalComplianceQueryResultBinding,
> {
    let holder = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        EstateLegalComplianceQuery,
        Principal,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(Principal::reference())
    .field(authority_holder_identity());
    let authority = ApplicationQueryResultShapeBuilder::<
        BankSchema,
        EstateLegalComplianceQuery,
        LegalAuthority,
        (),
        crate::queries::UnitQueryResultBinding,
    >::new(LegalAuthority::reference())
    .field(authority_identity())
    .field(authority_kind())
    .field(authority_recognized())
    .relation(authority_holder(), holder);
    ApplicationQueryResultShapeBuilder::new(EstateCase::reference())
        .field(estate_identity())
        .relation(estate_authorities(), authority)
        .build()
}
