//! Preserve the actual current-control refusal at an output source handoff.
use super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind as Kind};
use crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial as Cause;

pub trait OutputDemandDenialHistoryExt {
    fn history_source(cause: Cause) -> Self;
}

impl OutputDemandDenialHistoryExt for WorthQueryOutputDemandDenial {
    fn history_source(cause: Cause) -> Self {
        let kind = match cause {
            Cause::Authorization(stop) => Kind::of_request_authorization(stop.kind()),
            Cause::Stopped(stop) => Kind::ExecutionRequest(stop),
            Cause::Document(
                crate::domain_computation::primary_graph::WorthQueryDocumentHistoryDenial::Stopped(
                    stop,
                ),
            ) => Kind::ExecutionRequest(stop),
            Cause::ForeignDocument => Kind::ForeignDemand,
            Cause::ProductAdmission(cause) => {
                return Self::product_selection(cause, "history control")
            }
            _ => Kind::Superseded,
        };
        Self::new(kind, "history source handoff")
    }
}
