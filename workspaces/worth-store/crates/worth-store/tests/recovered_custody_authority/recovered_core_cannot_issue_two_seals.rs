use worth_store::physical_runtime::RecoveredPhysicalRuntimeCore;

fn duplicate(core: RecoveredPhysicalRuntimeCore) {
    let _first = core.into_serving_handoff();
    let _second = core.into_serving_handoff();
}

fn main() {}
