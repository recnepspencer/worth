use worth_proof::Recipe;

fn invalid_unresolved_bridge() {
    let unresolved = Recipe::new("payload");
    let _bridged = unresolved.bridge_trust_boundary();
}

fn main() {}
