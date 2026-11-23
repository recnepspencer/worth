use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

use crate::{estate::EstateCaseId, reads::EstateLegalAuthorityView, schema::BankSchema};

use super::{legal_compliance::EstateLegalComplianceQuery, legal_compliance_rows::EstateRow};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EstateLegalComplianceResult {
    estate: EstateCaseId,
    authorities: Vec<EstateLegalAuthorityView>,
}

impl EstateLegalComplianceResult {
    pub const fn estate(&self) -> EstateCaseId {
        self.estate
    }

    pub fn authorities(&self) -> &[EstateLegalAuthorityView] {
        &self.authorities
    }
}

impl WorthQueryApplicationProjection<BankSchema, EstateLegalComplianceQuery>
    for EstateLegalComplianceResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, BankSchema, EstateLegalComplianceQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let EstateRow {
            estate,
            authorities,
        } = EstateRow::project_row(row)?;
        Ok(Self {
            estate,
            authorities: authorities.into_iter().map(Into::into).collect(),
        })
    }
}
