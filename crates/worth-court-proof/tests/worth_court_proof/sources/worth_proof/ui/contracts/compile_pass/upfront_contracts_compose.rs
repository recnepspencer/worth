use std::convert::Infallible;
use worth_proof::contracts::*;

mod owner {
    worth_proof::authority_marker!(pub Authority);
    pub fn witness() -> worth_proof::AuthorityWitness<Authority> { Authority::witness() }
}

struct Source;
impl FreshnessSource for Source {
    type Sample = u64;
    type Error = Infallible;
    fn sample(&self) -> Result<u64, Infallible> { Ok(1) }
}

struct Policy;
impl FreshnessPolicy<Source, u64> for Policy {
    fn classify(&self, basis: &u64, now: &u64) -> FreshnessVerdict {
        if now <= basis { FreshnessVerdict::Current } else { FreshnessVerdict::RebindRequired }
    }
}

struct Action;
impl ActionMarker for Action {}

worth_proof::binding_axes! {
    struct RuntimeBinding {
        runtime: u64 => Runtime,
    }
    drift enum RuntimeBindingDrift;
}

fn main() {
    let _ = with_brand(|brand| brand.bind(1_u8).into_value());
    let _ = evaluate_freshness(1_u64, &Source, &Policy).unwrap();
    let _ = Performed::<Action, owner::Authority>::record(&owner::witness(), ());
    let comparison: () = Binding::new(RuntimeBinding { runtime: 1 })
        .ensure_matches(&Binding::new(RuntimeBinding { runtime: 1 }))
        .unwrap();
    let _ = comparison;
}
