use super::high::{Plain, Thing};

pub(crate) fn uses(thing: &Thing, plain: Plain) -> usize {
    thing.size() + plain.0
}
