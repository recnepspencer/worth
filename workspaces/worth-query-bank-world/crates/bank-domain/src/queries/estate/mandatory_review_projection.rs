use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

use crate::{estate::EstateCaseId, reads::EstateReviewView, schema::BankSchema};

use super::{mandatory_review::EstateMandatoryReviewQuery, mandatory_review_rows::EstateRow};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EstateMandatoryReviewResult {
    estate: EstateCaseId,
    reviews: Vec<EstateReviewView>,
}

impl EstateMandatoryReviewResult {
    pub const fn estate(&self) -> EstateCaseId {
        self.estate
    }

    pub fn reviews(&self) -> &[EstateReviewView] {
        &self.reviews
    }
}

impl WorthQueryApplicationProjection<BankSchema, EstateMandatoryReviewQuery>
    for EstateMandatoryReviewResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, BankSchema, EstateMandatoryReviewQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let EstateRow { estate, reviews } = EstateRow::project_row(row)?;
        Ok(Self {
            estate,
            reviews: reviews.into_iter().map(Into::into).collect(),
        })
    }
}
