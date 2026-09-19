use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;

/// The exact value the branch program activation record stores for one
/// canonical program revision.
///
/// The rendering travels one way only. A stored rendering is compared against
/// the renderings of an already admitted support roster to resolve which
/// rostered program an occurrence runs under; no path anywhere rebuilds an
/// [`ApplicationProgramRevision`] out of stored bytes.
pub(in crate::domain_computation::primary_graph) fn program_revision_rendering(
    revision: &ApplicationProgramRevision,
) -> AspectValue {
    AspectValue::String(InternedString::from(revision.to_string().as_str()))
}
