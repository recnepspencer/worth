use std::convert::Infallible;
use worth_proof::{
    evaluate_freshness, CurrentValidity, EvaluatedFreshness, FreshnessEvaluation, FreshnessPolicy,
    FreshnessScopedBasis, FreshnessSource, FreshnessVerdict,
};

struct OwnerSource;
struct CallerSource;

impl FreshnessSource for OwnerSource {
    type Sample = u64;
    type Error = Infallible;
    fn sample(&self) -> Result<u64, Infallible> { Ok(1) }
}

impl FreshnessSource for CallerSource {
    type Sample = u64;
    type Error = Infallible;
    fn sample(&self) -> Result<u64, Infallible> { Ok(0) }
}

struct OwnerPolicy;
struct CallerPolicy;

impl FreshnessPolicy<OwnerSource, ()> for OwnerPolicy {
    fn classify(&self, _: &(), _: &u64) -> FreshnessVerdict { FreshnessVerdict::Current }
}

impl FreshnessPolicy<CallerSource, ()> for CallerPolicy {
    fn classify(&self, _: &(), _: &u64) -> FreshnessVerdict { FreshnessVerdict::Current }
}

type OwnerCurrent = FreshnessScopedBasis<
    CurrentValidity,
    FreshnessEvaluation<OwnerSource, OwnerPolicy, ()>,
>;

fn owner_only(_: OwnerCurrent) {}

fn main() {
    let EvaluatedFreshness::Current(forged) =
        evaluate_freshness((), &CallerSource, &CallerPolicy).unwrap()
    else { return };
    owner_only(forged);
}
