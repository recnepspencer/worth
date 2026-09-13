/// Arena storage uses 64-element copy-on-write granules after an exact fork.
pub(crate) type PersistentPagedVector<T> = crate::data::persistent_vector::PersistentVector<T, 64>;
