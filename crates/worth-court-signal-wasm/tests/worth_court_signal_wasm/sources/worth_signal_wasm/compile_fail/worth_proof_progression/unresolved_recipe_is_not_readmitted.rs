use worth_proof::{Admitted, Recipe, Unresolved};

fn requires_readmitted_boundary_envelope(_: Recipe<Admitted, &'static str>) {}

fn main() {
    let bridged_but_unreadmitted_envelope = Recipe::<Unresolved, _>::new("worker boundary envelope");

    requires_readmitted_boundary_envelope(bridged_but_unreadmitted_envelope);
}
