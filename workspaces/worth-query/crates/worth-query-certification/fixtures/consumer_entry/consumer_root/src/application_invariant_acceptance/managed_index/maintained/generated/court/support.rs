//! Row reads, the literal expected universe and the suspended-read cause shared by the generated court.
use super::*;

pub(super) fn find(directory: &str) -> query::FindPrefix {
    query::FindPrefix {
        directory: directory.into(),
        prefix: "taken".into(),
    }
}
pub(super) fn values(rows: &[query::ItemRow]) -> Vec<(String, String)> {
    let mut rows: Vec<_> = rows
        .iter()
        .map(|row| (row.identity.clone(), row.name.clone()))
        .collect();
    rows.sort();
    rows
}
// A literal authored universe independent of provider nodes and Source lookup.
pub(super) fn expected(directory: &str, name: &str) -> Vec<(String, String)> {
    let mut rows = vec![("generated-item".into(), name.into())];
    if directory == "B" {
        rows.push(("y".into(), "taken".into()));
    }
    rows.sort();
    rows
}
pub(super) fn assert_suspended<T>(result: Result<T, WorthQueryApplicationRequestQueryDenial>) {
    match result {
        Err(WorthQueryApplicationRequestQueryDenial::PrincipalResolution(denial)) => assert_eq!(
            denial.kind(),
            WorthQueryPrincipalResolutionDenialKind::Refused(
                WorthQueryPrincipalResolutionRefusalKind::BranchMaterializationSuspended
            )
        ),
        Err(denial) => panic!("wrong current read cause: {denial:?}"),
        Ok(_) => panic!("a current physical read cannot expose retained logical material"),
    }
}
