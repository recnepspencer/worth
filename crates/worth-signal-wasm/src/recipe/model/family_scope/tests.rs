use super::*;
use worth_signal::facade::ScopeCoverage;

#[test]
fn family_wire_scope_resolves_keyed_detail_and_explicit_subtree() {
    let exact: RecipeFamilyReadScopeSpec = serde_json::from_value(serde_json::json!({
        "partitionFrom": "key", "detail": "5y"
    }))
    .unwrap();
    let exact = exact.resolve("rates").unwrap();
    assert_eq!(exact.path().segments(), &["rates", "5y"]);
    assert_eq!(exact.coverage(), ScopeCoverage::Exact);

    let subtree: RecipeFamilyReadScopeSpec = serde_json::from_value(serde_json::json!({
        "partition": "rates", "detail": "ignored", "matchMode": "WholePartition"
    }))
    .unwrap();
    let subtree = subtree.resolve("unused").unwrap();
    assert_eq!(subtree.path().segments(), &["rates"]);
    assert_eq!(subtree.coverage(), ScopeCoverage::Subtree);
}

#[test]
fn invalid_family_scope_is_denied_instead_of_becoming_an_unscoped_read() {
    for wire in [
        serde_json::json!({}),
        serde_json::json!({"partitionFrom": "unsupported"}),
        serde_json::json!({"partitionFrom": "key"}),
        serde_json::json!({"partition": "rates", "detail": ""}),
        serde_json::json!({"partition": "rates", "matchMode": "PartitionAndDetail"}),
    ] {
        let scope: RecipeFamilyReadScopeSpec = serde_json::from_value(wire).unwrap();
        assert_eq!(scope.resolve("").unwrap_err().code, "invalidInput");
    }
}

#[test]
fn direct_recipe_wire_retains_all_eight_opaque_segments_and_rejects_nine() {
    let segments: Vec<_> = (0..8).map(|level| format!("segment:{level}")).collect();
    let wire = serde_json::json!({
        "id": "source", "scope": {"path": segments, "coverage": "Exact"}
    });
    let read: crate::recipe::model::RecipeReadSignalSpec = serde_json::from_value(wire).unwrap();
    let scope = read.scope.unwrap();
    assert_eq!(scope.path().segments(), segments);
    assert_eq!(scope.coverage(), ScopeCoverage::Exact);
    let too_deep = serde_json::json!({
        "id": "source", "scope": {"path": ["a","b","c","d","e","f","g","h","i"], "coverage": "Subtree"}
    });
    assert!(
        serde_json::from_value::<crate::recipe::model::RecipeReadSignalSpec>(too_deep).is_err()
    );
}
