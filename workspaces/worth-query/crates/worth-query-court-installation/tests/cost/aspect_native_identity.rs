use crate::installation::aspect_native_identity::model;

fn measure<const R: usize, const F: usize>() -> (u32, usize) {
    let declaration = model::declaration::<R, F>(false).unwrap();
    let admitted = model::package(&declaration).validate().unwrap();
    let source = admitted.canonical_work();
    let index = model::index_from(admitted);
    let operation = index
        .bind_application_schema(declaration)
        .unwrap()
        .installed_operation(model::operation())
        .expect("finite aspect-native declaration installs");
    let work = operation
        .graph_obligations()
        .installation_evidence()
        .canonical_work();
    assert!(work.canonical_entries() <= 16 * source.canonical_entries());
    assert!(work.canonical_encoded_bytes() <= 16 * source.canonical_encoded_bytes());
    println!(
        "aspect-native records={R} fields={F} entries={} bytes={}",
        work.canonical_entries(),
        work.canonical_encoded_bytes()
    );
    (work.canonical_entries(), work.canonical_encoded_bytes())
}

#[test]
fn identity_growth_tracks_distinct_contracts_and_field_facts() {
    let narrow = [
        measure::<2, 1>().0,
        measure::<4, 1>().0,
        measure::<8, 1>().0,
    ];
    let three = [
        measure::<2, 3>().0,
        measure::<4, 3>().0,
        measure::<8, 3>().0,
    ];
    let six = [
        measure::<2, 6>().0,
        measure::<4, 6>().0,
        measure::<8, 6>().0,
    ];
    let nine = [
        measure::<2, 9>().0,
        measure::<4, 9>().0,
        measure::<8, 9>().0,
    ];
    for counts in [narrow, three, six, nine] {
        assert!(counts[1] < counts[0] * 3);
        assert!(counts[2] < counts[1] * 3);
    }
    // Equal field increments must have comparable cost. Re-embedding a growing
    // aspect at every field write makes the later increment substantially larger.
    for index in 0..3 {
        let early = six[index] - three[index];
        let late = nine[index] - six[index];
        assert!(late * 10 < early * 12, "field growth must stay linear");
    }
}
