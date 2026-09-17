use worth_query_execution::facade::primary_graph::WorthQueryAdmittedDisclosedApplicationResult;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationOutputDemandDisclosure, WorthQueryApplicationOutputDemandSource,
};

mod basis;
mod disclosure;
mod inspection;
mod receipt;
mod terminal_release;

pub use basis::{WorthQueryPublishedApplicationBasis, WorthQueryPublishedApplicationBasisPosture};
pub use disclosure::{
    WorthQueryPublishedApplicationDisclosure, WorthQueryPublishedApplicationDisclosureIdentity,
    WorthQueryPublishedApplicationDisclosurePosture,
};
pub use inspection::WorthQueryApplicationQueryPublicationInspection;
pub use receipt::{
    WorthQueryApplicationQueryPublicationReceipt,
    WorthQueryPublishedApplicationQueryOmissionPosture,
};
pub use terminal_release::{
    WorthQueryPublishedApplicationQueryReleasePosture,
    WorthQueryPublishedApplicationQueryResultBufferRelease,
    WorthQueryPublishedApplicationQueryTerminalRelease,
};

/// Publication-owned result whose input was already governed before domain
/// projection. Publication performs no field-policy decision or redaction.
pub struct WorthQueryPublishedApplicationResult<Query, QueryResult> {
    source: WorthQueryApplicationOutputDemandSource<Query, QueryResult>,
    receipt: WorthQueryApplicationQueryPublicationReceipt,
}

/// Accepts only Query's admitted disclosed shape.
///
/// Raw application values cannot enter publication:
///
/// ```compile_fail
/// use worth_query_publication::facade::domain_computation::publish_application_result;
///
/// let raw = vec!["protected".to_string()];
/// let _ = publish_application_result::<(), _>(raw);
/// ```
///
/// A descriptive Foundational mask is not a publication result:
///
/// ```compile_fail
/// use worth_foundational::facade::{AspectMask, ProjectionMask};
/// use worth_query_publication::facade::domain_computation::publish_application_result;
///
/// let mask = AspectMask::<ProjectionMask>::whole_aspect();
/// let _ = publish_application_result::<(), ()>(mask);
/// ```
///
/// A diagnostic mask is equally incapable of opening publication:
///
/// ```compile_fail
/// use worth_foundational::facade::{AspectMask, DiagnosticMask};
/// use worth_query_publication::facade::domain_computation::publish_application_result;
///
/// let diagnostic = AspectMask::<DiagnosticMask>::whole_aspect();
/// let _ = publish_application_result::<(), ()>(diagnostic);
/// ```
pub fn publish_application_result<Query, QueryResult>(
    admitted: WorthQueryAdmittedDisclosedApplicationResult<Query, QueryResult>,
) -> WorthQueryPublishedApplicationResult<Query, QueryResult> {
    let receipt = WorthQueryApplicationQueryPublicationReceipt::from_terminal(admitted.receipt());
    let source = admitted.into_output_demand_source();
    WorthQueryPublishedApplicationResult { source, receipt }
}

impl<Query, QueryResult> WorthQueryPublishedApplicationResult<Query, QueryResult> {
    pub fn rows(&self) -> &[QueryResult] {
        self.source.rows()
    }

    pub const fn receipt(&self) -> &WorthQueryApplicationQueryPublicationReceipt {
        &self.receipt
    }

    pub fn observed_sources(
        &self,
    ) -> &[worth_query_execution::facade::primary_graph::WorthQueryObservedSource<Query>] {
        self.source.observed_sources()
    }

    pub fn into_output_demand_disclosure(
        self,
    ) -> WorthQueryApplicationOutputDemandDisclosure<Query> {
        self.source.into_disclosure()
    }

    pub fn into_rows(self) -> Vec<QueryResult> {
        self.source.into_rows()
    }

    pub(crate) fn into_output_demand_source(
        self,
    ) -> WorthQueryApplicationOutputDemandSource<Query, QueryResult> {
        self.source
    }
}
