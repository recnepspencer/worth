//! Explicit optional authored roles; ordinary extra Retire roles remain distinct.
use super::{lifecycle::*, *};
pub struct CoCreated<S, const I: usize>(PhantomData<fn() -> S>);
pub struct CoRetired<S, const I: usize>(PhantomData<fn() -> S>);
impl<S: Scenario, const I: usize> WorthQueryApplicationOutputRole for CoCreated<S, I> {
    type Schema = S;
    type Contract = CreatedOutputs;
    type Entity = Node;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = ["co-created-a", "co-created-b"][I];
}
impl<S: Scenario, const I: usize> WorthQueryApplicationOutputRole for CoRetired<S, I> {
    type Schema = S;
    type Contract = RetiredOutputs;
    type Entity = Node;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = ["co-retired-a", "co-retired-b"][I];
}
