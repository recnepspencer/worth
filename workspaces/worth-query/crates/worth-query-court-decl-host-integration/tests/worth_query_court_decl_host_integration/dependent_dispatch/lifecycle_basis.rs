//! A legitimate self-superseding producer owns a separate auxiliary record.
use super::split_support::*;
use super::*;
#[test]
fn dependent_owned_absence_selects_preserve_without_mutating_source() {
    // root Initial; child Initial fills its OWN cell's absent aspect, publishes,
    // then child Preserve; leaf Initial. The parent never owns or witnesses cell.
    let selections = Selections::default();
    let app = world::install_with_cell(1, selections.clone(), true);
    let (scope, principal) = world::authenticate(&app);
    let request = app.request(&principal, &scope);
    let settled = settle(&app, &request);
    sequence(&selections, "root", &[(false, false)]);
    sequence(&selections, "root.0", &[(false, false), (true, true)]);
    sequence(&selections, "root.0.0", &[(false, false)]);
    assert_eq!(value(&request, "root.0"), 12);
    assert_eq!(value(&request, "root.0.0"), 13);
    close(settled, &request);
}
