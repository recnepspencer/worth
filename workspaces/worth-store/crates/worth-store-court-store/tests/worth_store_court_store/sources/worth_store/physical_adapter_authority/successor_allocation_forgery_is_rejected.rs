use worth_store::physical_runtime::RecoveryPhysicalAllocation;

fn unavailable<T>() -> T {
    loop {
        std::hint::spin_loop();
    }
}

fn forge() -> RecoveryPhysicalAllocation<'static> {
    RecoveryPhysicalAllocation {
        allocation: unavailable(),
    }
}

fn main() {}
