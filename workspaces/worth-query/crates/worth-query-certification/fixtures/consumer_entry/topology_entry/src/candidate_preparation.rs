//! The guide's declaration-only contract and finite prepared footprint through public facades.
// compiled-example: begin
use worth_query_decl::facade::application_operation::{
    ApplicationCandidateCardinalityCeiling, ApplicationCandidateContract,
    ApplicationCandidateRequirements, ApplicationCandidateResourceCeiling,
};
const CANDIDATES: ApplicationCandidateContract = ApplicationCandidateContract::prepared();
fn prepared_footprint(
    creates: usize,
    writes: usize,
    owned_bytes: usize,
) -> ApplicationCandidateRequirements {
    ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(creates, 0, 0, 0, writes, 0),
        ApplicationCandidateResourceCeiling::representation_bytes(owned_bytes),
    )
}
// compiled-example: end

pub fn declaration_and_footprint(
    creates: usize,
    writes: usize,
    owned_bytes: usize,
) -> (
    ApplicationCandidateContract,
    ApplicationCandidateRequirements,
) {
    (CANDIDATES, prepared_footprint(creates, writes, owned_bytes))
}
