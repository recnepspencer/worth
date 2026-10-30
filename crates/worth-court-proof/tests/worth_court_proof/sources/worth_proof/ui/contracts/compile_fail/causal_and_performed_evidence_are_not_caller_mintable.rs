use std::marker::PhantomData;
use worth_proof::{DerivedFrom, Inverts, Performed};

worth_proof::authority_marker!(pub RuntimeAuthority);

struct Action;
impl worth_proof::ActionMarker for Action {}

fn main() {
    let _performed = Performed::<Action, RuntimeAuthority> {
        outcome: (),
        action: PhantomData,
        authority: PhantomData,
    };
    let _derived = DerivedFrom::<Action, RuntimeAuthority> {
        predecessor: PhantomData,
        authority: PhantomData,
    };
    let _inverts = Inverts::<Action, RuntimeAuthority> {
        original: PhantomData,
        authority: PhantomData,
    };
}
