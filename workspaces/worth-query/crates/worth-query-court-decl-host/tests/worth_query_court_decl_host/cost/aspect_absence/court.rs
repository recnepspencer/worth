use super::*;
#[test]
fn selected_present_and_absent_reads_are_flat_but_class_discovery_grows() {
    let observed = [
        timed(3, three::measure),
        timed(6, six::measure),
        timed(12, twelve::measure),
    ];
    // Literal roster and expected payload are independent of Query's counters.
    for (classes, [present, absent, discovery]) in [3, 6, 12].into_iter().zip(observed) {
        assert_eq!(present.field_reads(), 2, "payload plus sink; {classes}");
        assert_eq!(
            absent.field_reads(),
            1,
            "one acquired absent value; {classes}"
        );
        assert_eq!(
            discovery.field_reads(),
            classes + 1,
            "all alternatives plus sink"
        );
        assert_eq!(present.equality_lookups(), 1);
        assert_eq!(absent.equality_lookups(), 1);
        assert_eq!(discovery.equality_lookups(), 1);
        println!("classes={classes} selected-present={present:?} selected-absent={absent:?} exhaustive={discovery:?}");
    }
}

fn timed(
    classes: usize,
    run: fn() -> [WorthQueryInvariantProjectionWork; 3],
) -> [WorthQueryInvariantProjectionWork; 3] {
    let start = std::time::Instant::now();
    let result = run();
    println!(
        "classes={classes} three-installs-operations-and-readbacks={:?}",
        start.elapsed()
    );
    result
}
