mod interest;
mod program_interest;
mod request;
mod settlement;

pub(in crate::application_entry) use program_interest::{
    WorthQueryApplicationProgramDemandHandle, WorthQueryApplicationProgramDemandProgress,
};

pub use interest::WorthQueryApplicationOutputDemandHandle;
pub use request::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationOutputDemandRequest,
    WorthQueryOutputDemandControls,
};
pub use settlement::{
    WorthQueryApplicationOutputDemandProgress, WorthQueryApplicationOutputDemandSettlement,
    WorthQueryOutputSettlementPosture,
};

impl WorthQueryApplicationOutputDemandDenial {
    pub(in crate::application_entry) fn advancement(
        cause: worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
    ) -> Self {
        Self::Demand(worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenial::request_admission(cause))
    }
}
