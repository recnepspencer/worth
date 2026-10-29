//! A capture policy that does not descend from the funding lease refuses at
//! entry: no authoring, no commit, no charge, and the checkpoint unchanged.
//! A foreign lease is nested-lease misuse; naming no lease is unleased backing.
pub use super::*;
pub use worth_execution::ExecutionResourceLease;
pub use worth_query_host::facade::{
    application_contribution::WorthQueryManagedComputationResourceDenial as Resource,
    primary_graph::{
        WorthQueryPrimaryGraphInstallationDenialKind as Kind,
        WorthQueryProviderSessionDenialKind as SessionKind,
    },
};

pub fn assert_refused_at_entry(
    denied: Result<Target, installation::WorthQueryInMemoryApplicationDenial>,
    unleased: bool,
) {
    let installation::WorthQueryInMemoryApplicationDenial::Graph(denial) = denied
        .err()
        .expect("a capture outside the funding lineage must refuse at entry")
    else {
        panic!("the refusal is the typed entry refusal, not a later stage")
    };
    if unleased {
        assert!(
            matches!(
                denial.kind(),
                Kind::ExecutionDenied {
                    kind: SessionKind::ResourceEnvelopeMismatch
                }
            ),
            "{denial:?}"
        );
    } else {
        assert!(
            matches!(
                denial.kind(),
                Kind::ExecutionDenied {
                    kind: SessionKind::ExecutionResource {
                        denial: Resource::NestedLeaseMisuse,
                        ..
                    }
                }
            ),
            "{denial:?}"
        );
    }
}
