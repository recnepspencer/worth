struct Identity { family: std::any::TypeId }
type Key = Identity;
struct Registry { entries: std::collections::BTreeMap<Key, ()> }
