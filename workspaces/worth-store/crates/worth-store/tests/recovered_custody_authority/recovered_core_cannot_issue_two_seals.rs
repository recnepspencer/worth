use worth_store::physical_runtime::RecoveredPhysicalRuntimeCore;

fn duplicate(core: RecoveredPhysicalRuntimeCore) {
    let _first = core.into_checkpoint_custody();
    let _second = core.into_checkpoint_custody();
}

fn main() {}
