//! Governed disclosure, effect-driven live mode and namespace members lower
//! as declared, and their malformed forms are refused at the offending key.
use super::expand;

const HEADER: &str = "#[worth_query(schema = S, context = C, namespace = NS, \
     result = Row, root = Project, scope = ProjectId, limits = 1)]";
const FIELD: &str = "{ #[worth_query(scope)] project: String }";
const LIVE: &str =
    "live = effect(Renamed, row_key = Row::id(), payload_key = |r| r.project.clone())";

fn expansion(source: &str) -> Result<String, String> {
    let input = syn::parse_str(source).expect("the court input parses");
    expand(&input)
        .map(|tokens| tokens.to_string())
        .map_err(|error| error.to_string())
}

#[test]
fn malformed_governed_and_live_queries_are_refused() {
    let cases = [
        (
            format!("#[worth_query(disclose_by = access::scope_view())] {HEADER} struct Read {FIELD}"),
            "`disclose_by` does not apply to a query without a governed `disclosure = ..`",
        ),
        (
            format!("#[worth_query(live = effect(Renamed, row_key = Row::id()))] {HEADER} struct Read {FIELD}"),
            "requires `payload_key`",
        ),
        (
            format!("#[worth_query(live = effect(Renamed, rows = Row::id()))] {HEADER} struct Read {FIELD}"),
            "unknown `effect(..)` key `rows`",
        ),
        (
            format!("#[worth_query({LIVE}, lanes = OneShot)] {HEADER} struct Read {FIELD}"),
            "`live` requires live lane eligibility",
        ),
    ];
    for (source, expected) in cases {
        let refusal = expansion(&source).expect_err("the query is refused");
        assert!(refusal.contains(expected), "{source}\n=> {refusal}");
    }
}

#[test]
fn governed_live_query_lowers_disclosure_cause_and_intent() {
    let source = format!(
        "#[worth_query(disclosure = access::disclosure(), disclose_by = access::scope_view(), {LIVE})] \
         {HEADER} pub struct Read {FIELD}"
    );
    let lowered = expansion(&source).expect("the query lowers");
    for expected in [
        "use_field_by (< ProjectId > :: at (\"ProjectId\") , access :: scope_view ()",
        "disclose_result_by (access :: scope_view ()",
        "live_root_by :: < ReadLiveCause > (Row :: id () , Row :: id ())",
        "one_shot () . with_live ()",
        "worth_query_identity ! (NS , live_cause)",
        "ApplicationLiveQueryIntent",
        "type LiveCause = ReadLiveCause",
    ] {
        assert!(
            lowered.contains(expected),
            "missing `{expected}` in\n{lowered}"
        );
    }
}

#[test]
fn a_member_narrows_every_identity_to_its_segment() {
    let derived = format!("#[worth_query(member)] {HEADER} pub struct ReadShared {FIELD}");
    let explicit =
        format!("#[worth_query(member = \"read\")] {HEADER} pub struct ReadShared {FIELD}");
    let derived = expansion(&derived).expect("the query lowers");
    let explicit = expansion(&explicit).expect("the query lowers");
    assert!(derived.contains("worth_query_identity_member ! (NS , \"read_shared\")"));
    assert!(explicit.contains("worth_query_identity_member ! (NS , \"read\")"));
    assert!(!derived.contains("worth_query_identity ! (NS , query)"));
}
