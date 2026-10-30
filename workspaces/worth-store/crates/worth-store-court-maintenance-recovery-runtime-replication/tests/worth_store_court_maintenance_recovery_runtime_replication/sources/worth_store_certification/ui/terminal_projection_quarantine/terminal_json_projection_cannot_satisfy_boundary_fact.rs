use worth_store_aspect_native::{
    StoreAspectBoundaryFact, StoreTerminalJsonProjection,
};

fn require_boundary_fact(_: StoreAspectBoundaryFact) {}

fn main() {
    let projection: StoreTerminalJsonProjection = todo!();
    require_boundary_fact(projection);
}
