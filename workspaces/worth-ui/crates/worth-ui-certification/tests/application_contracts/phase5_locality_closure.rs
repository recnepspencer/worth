use std::collections::BTreeSet;
use worth_ui_certification::scenario::phase5_locality_matrix::execute_local_closure;

use crate::phase5_locality_worker::invocation;

const EVIDENCE_PREFIX: &str = "WORTH_UI_PHASE5_PRODUCTION_LOCALITY=";
#[test]
#[ignore = "closure portfolio: 32 fresh native worlds use controlled isolated shards"]
fn all_32_fresh_native_locality_worlds_retain_owner_issued_evidence() {
    let (executable, arguments) = invocation();
    let rows = execute_local_closure(&executable, &arguments)
        .expect("the Phase 5 locality matrix completes");
    println!(
        "{EVIDENCE_PREFIX}{}",
        serde_json::to_string(&rows).expect("locality evidence is valid JSON")
    );
    assert_eq!(rows.len(), 32);

    let cases = rows
        .iter()
        .map(|row| {
            assert_eq!(row["terminal_zero"], true);
            let retained = row["retained"].as_u64().expect("retained size");
            let paragraphs = row["retained_paragraphs"]
                .as_u64()
                .expect("retained paragraph count");
            let mechanics = row["retained_mechanics"]
                .as_u64()
                .expect("retained mechanic count");
            let initial_mechanics = row["text_work"][0]["layouts"]
                .as_u64()
                .expect("initial retained text mechanics");
            assert_eq!(mechanics, if retained == 1 { 2 } else { retained });
            assert_eq!(paragraphs, if retained == 1 { 1 } else { retained / 2 });
            assert_eq!(
                initial_mechanics, mechanics,
                "matrix scale must reach the qualified retained-mechanic ceiling",
            );
            assert!(row["query_completed"]
                .as_u64()
                .is_some_and(|count| count >= 2));
            assert!(row["physical_signal"]["performed_nodes"]
                .as_u64()
                .is_some_and(|performed| performed > 0));
            assert!(row["world_elapsed_ms"].as_u64().is_some());
            for phase in [
                "profile",
                "platform_prepare",
                "query_install",
                "fixture_materialization",
                "owner_installation",
                "builder_registration",
                "application_completion",
                "native_run",
            ] {
                assert!(
                    row["timing_us"][phase].as_u64().is_some(),
                    "matrix row omits timing phase {phase}"
                );
            }
            let axis = row["axis"].as_str().expect("locality axis");
            (retained, axis.to_owned())
        })
        .collect::<BTreeSet<_>>();
    let expected = [1_u64, 32, 2_048, 4_096]
        .into_iter()
        .flat_map(|retained| {
            [
                "content",
                "width",
                "paint-value",
                "paint-boundary",
                "dpi",
                "atlas-miss",
                "upload-completion",
                "pin-release",
            ]
            .into_iter()
            .map(move |axis| (retained, axis.to_owned()))
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(cases, expected);
}
