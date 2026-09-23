use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use crate::product_process::{CargoBuiltPlatformPulse, SuccessfulPlatformPulseExit};

#[test]
#[ignore = "requires the serialized interactive certified native desktop (Windows 11 DX12 or Xvfb X11)"]
fn live_layout_pins_cross_runtime_native_signal_and_release_at_last_owner() {
    let mut launch = CargoBuiltPlatformPulse::exact()
        .and_then(CargoBuiltPlatformPulse::launch_native_gate_d_pin_world)
        .expect("the Gate D pin product world launches under the desktop lease");
    let deadline = Instant::now() + Duration::from_secs(120);
    let exit = SuccessfulPlatformPulseExit::wait(&mut launch.process, deadline)
        .expect("the Gate D pin product world closes without residue");
    assert!(exit.status().success());
    let mut stdout = String::new();
    launch
        .stdout
        .read_to_string(&mut stdout)
        .expect("the Gate D pin evidence is readable");
    let evidence: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("the Gate D pin evidence is exact JSON");
    assert_eq!(evidence["schema"], "worth-ui-native-gate-d-pin-world-v3");
    assert_eq!(evidence["mounted_bindings"], 1);
    assert_eq!(evidence["pinned_layouts"], 3);
    assert_eq!(evidence["presentations"], 4);
    assert_eq!(evidence["atlas_transactions"], 4);
    assert_eq!(evidence["native_peak_pin_count"], 49);
    let frame_pins = evidence["native_frame_pin_counts"].as_array().unwrap();
    assert_eq!(frame_pins, &[49, 49, 40, 0]);
    assert_exact_layout_key_transition(&evidence);
    assert_eq!(evidence["physical_signal_runtimes"], 1);
    assert_eq!(evidence["physical_signal_workers"], 1);
    assert_eq!(evidence["alpha_entries"], 40);
    assert_eq!(evidence["color_entries"], 1);
    assert_eq!(evidence["query_close_complete"], true);
    assert!(evidence["closed_query_resources"].as_u64().unwrap() > 0);
    assert_eq!(evidence["terminal_zero"], true);
}

fn assert_exact_layout_key_transition(evidence: &serde_json::Value) {
    let frames = evidence["native_frame_pins"].as_array().unwrap();
    assert_eq!(frames.len(), 4);
    let first = layout_keys(&frames[0]);
    let second = layout_keys(&frames[1]);
    let third = layout_keys(&frames[2]);
    assert_eq!(first.len(), 3);
    assert_eq!(first, second);
    assert_eq!(third.len(), 2);
    assert!(frames[3].as_array().unwrap().is_empty());
    for (layout, keys) in &third {
        assert_eq!(first.get(layout), Some(keys));
    }
    let removed = first
        .iter()
        .find(|(layout, _)| !third.contains_key(*layout))
        .map(|(_, keys)| keys)
        .expect("one mounted text layout is removed");
    let shared_survivor = third
        .values()
        .find(|keys| !keys.is_disjoint(removed))
        .expect("one retained layout shares raster keys with the removed layout");
    assert!(!removed
        .difference(shared_survivor)
        .collect::<Vec<_>>()
        .is_empty());
    assert!(!shared_survivor
        .difference(removed)
        .collect::<Vec<_>>()
        .is_empty());
}

fn layout_keys(frame: &serde_json::Value) -> BTreeMap<&str, BTreeSet<&str>> {
    let mut layouts = BTreeMap::new();
    for pin in frame.as_array().unwrap() {
        layouts
            .entry(pin["layout"].as_str().unwrap())
            .or_insert_with(BTreeSet::new)
            .insert(pin["raster_key"].as_str().unwrap());
    }
    layouts
}
