// Literal retained-source growth under identical finite native page limits.
#[allow(unused_imports)]
use crate::court::reference_context::record_history::growth::*;

#[test]
fn ten_and_ten_thousand_unrelated_changes_keep_the_same_finite_page_work_and_storage() {
    let small = observe_growth(10);
    let large = observe_growth(10_000);
    assert_eq!(
        large, small,
        "page work and backing do not grow with retained source size"
    );
}
