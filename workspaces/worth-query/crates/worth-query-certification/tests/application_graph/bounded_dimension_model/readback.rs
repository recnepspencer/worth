//! What one occurrence actually says a part's dimension is.
//!
//! Every verdict in this court is checked against a committed read rather than
//! an outcome variant, so a denial that silently landed an effect, or a
//! performed commit that wrote nothing, fails here.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationReadObservation, WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_host::facade::product::WorthQueryProductBranch;

use super::dimension_entry::{PartDimensionRead, PART_IDENTITY};
use super::operator_identity::{authenticate_operator, request_scope};
use super::schema::BoundedDimensionSchema;

/// Reads the part's committed dimension on one exact occurrence.
pub fn read_dimension(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<BoundedDimensionSchema>,
    branch: WorthQueryProductBranch,
) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .query(PartDimensionRead {
            identity: PART_IDENTITY.to_owned(),
        })
        .execute()
        .expect("the ordinary read must execute on this occurrence")
        .rows()[0]
        .dimension
}

/// Retains this occurrence's published head, which is what a denial before
/// effects must leave exactly where it found it.
pub fn observe_head(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<BoundedDimensionSchema>,
    branch: WorthQueryProductBranch,
) -> WorthQueryApplicationReadObservation {
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .on_branch(branch)
        .retain_read()
        .expect("the occurrence must remain selectable")
}
