use worth_runtime_world::facade::{CompositeBasisKey, RuntimeWorldOwnerIdentity};

fn placeholder<T>() -> T {
    loop {}
}

fn issue(owner: RuntimeWorldOwnerIdentity) -> CompositeBasisKey {
    CompositeBasisKey::issued(
        owner,
        placeholder(),
        placeholder(),
        placeholder(),
    )
}

fn main() {}
