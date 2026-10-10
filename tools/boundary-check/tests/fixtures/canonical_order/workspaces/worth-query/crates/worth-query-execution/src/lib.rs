use std::any::TypeId;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
struct DemandKey {
    family: TypeId,
}
