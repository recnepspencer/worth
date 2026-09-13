mod reserved_fork;
#[path = "persistent_ord_set/retained_charge.rs"]
mod retained_charge;
#[cfg(test)]
#[path = "persistent_ord_set/retained_charge_tests.rs"]
mod retained_charge_tests;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::data::persistent_ord_map::PersistentOrdMap;

/// An ordered set with flat unique storage and element-granular fork changes.
#[derive(PartialEq, Eq)]
pub(crate) struct PersistentOrdSet<T: Clone + Ord> {
    values: PersistentOrdMap<T, ()>,
}

impl<T: Clone + Ord> PersistentOrdSet<T> {
    pub(crate) fn new() -> Self {
        Self {
            values: PersistentOrdMap::new(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.values.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub(crate) fn contains<Q>(&self, value: &Q) -> bool
    where
        T: std::borrow::Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.values.contains_key(value)
    }

    pub(crate) fn insert(&mut self, value: T) -> bool {
        self.values.insert(value, ()).is_none()
    }

    pub(crate) fn remove(&mut self, value: &T) -> bool {
        self.values.remove(value).is_some()
    }

    pub(crate) fn clear(&mut self) {
        self.values.clear();
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.values.keys()
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        self.values.ptr_eq(&other.values)
    }

    pub(crate) fn operational_clone(&self) -> Self {
        Self {
            values: self.values.operational_clone(),
        }
    }

    pub(crate) fn fork_persistent(&mut self) -> Self {
        Self {
            values: self.values.fork_persistent(),
        }
    }

    #[cfg(test)]
    pub(crate) fn fork_storage_identity(&self) -> Self {
        Self {
            values: self.values.fork_storage_identity(),
        }
    }
}

impl<T: Clone + Ord> Clone for PersistentOrdSet<T> {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
        }
    }
}

impl<T: Clone + Ord> FromIterator<T> for PersistentOrdSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            values: iter.into_iter().map(|value| (value, ())).collect(),
        }
    }
}

impl<T: Clone + Ord> Default for PersistentOrdSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Ord> Extend<T> for PersistentOrdSet<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.insert(value);
        }
    }
}

impl<T> std::fmt::Debug for PersistentOrdSet<T>
where
    T: Clone + Ord + std::fmt::Debug,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_set().entries(self.iter()).finish()
    }
}

impl<T> Serialize for PersistentOrdSet<T>
where
    T: Clone + Ord + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self.iter())
    }
}

impl<'de, T> Deserialize<'de> for PersistentOrdSet<T>
where
    T: Clone + Ord + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Vec::<T>::deserialize(deserializer)?.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::collections::BTreeSet;
    use std::env;
    use std::hint::black_box;
    use std::process::Command;

    use serde::Serialize;
    use stats_alloc::{Region, INSTRUMENTED_SYSTEM};

    use super::PersistentOrdSet;

    thread_local! {
        static ITEM_CLONES: Cell<usize> = const { Cell::new(0) };
    }

    #[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
    #[serde(transparent)]
    struct Counted(u64);

    impl Clone for Counted {
        fn clone(&self) -> Self {
            ITEM_CLONES.set(ITEM_CLONES.get() + 1);
            Self(self.0)
        }
    }

    #[test]
    fn fork_churn_and_readmission_follow_map_storage_without_history() {
        for churn_count in [64_u64, 4_096, 65_536] {
            let mut source = PersistentOrdSet::new();
            source.insert(0);
            let mut fork = source.fork_persistent();
            assert!(source.ptr_eq(&fork), "scale {churn_count} must share");

            for value in 1..=churn_count {
                assert!(fork.insert(value));
                assert!(fork.remove(&value));
            }
            assert_eq!(fork.iter().copied().collect::<Vec<_>>(), vec![0]);
            assert_eq!(source.iter().copied().collect::<Vec<_>>(), vec![0]);

            assert!(fork.remove(&0));
            assert!(fork.insert(0));
            assert_eq!(fork.iter().copied().collect::<Vec<_>>(), vec![0]);
            assert_eq!(source.iter().copied().collect::<Vec<_>>(), vec![0]);
        }
    }

    #[test]
    fn ordinary_serialization_borrows_values_without_a_temporary_collection() {
        const CHILD: &str = "WORTH_SIGNAL_SET_SERIALIZATION_COST_CHILD";
        const TEST: &str =
            "data::persistent_ord_set::tests::ordinary_serialization_borrows_values_without_a_temporary_collection";
        if env::var_os(CHILD).is_none() {
            let output = Command::new(env::current_exe().expect("test executable resolves"))
                .arg("--exact")
                .arg(TEST)
                .arg("--nocapture")
                .arg("--test-threads=1")
                .env(CHILD, "1")
                .output()
                .expect("isolated set-serialization probe starts");
            let stdout = String::from_utf8_lossy(&output.stdout);
            print!("{stdout}");
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
            assert!(output.status.success(), "set-serialization probe failed");
            assert!(
                stdout.contains(TEST) && stdout.contains("test result: ok. 1 passed; 0 failed;"),
                "set-serialization probe must execute exactly one named test"
            );
            return;
        }

        for value_count in [64_u64, 4_096, 65_536] {
            let values: PersistentOrdSet<Counted> = (0..value_count).map(Counted).collect();
            let native: BTreeSet<Counted> = (0..value_count).map(Counted).collect();

            ITEM_CLONES.set(0);
            let native_region = Region::new(&INSTRUMENTED_SYSTEM);
            let expected = black_box(serde_json::to_vec(&native).expect("native set serializes"));
            let native_allocation = native_region.change();
            assert_eq!(ITEM_CLONES.get(), 0);

            ITEM_CLONES.set(0);
            let actual_region = Region::new(&INSTRUMENTED_SYSTEM);
            let actual = black_box(serde_json::to_vec(&values).expect("persistent set serializes"));
            let actual_allocation = actual_region.change();

            assert_eq!(actual, expected, "scale {value_count} wire changed");
            assert_eq!(ITEM_CLONES.get(), 0, "serialization must borrow values");
            assert_eq!(
                actual_allocation.allocations, native_allocation.allocations,
                "scale {value_count} added serialization allocations"
            );
            assert_eq!(
                actual_allocation.bytes_allocated, native_allocation.bytes_allocated,
                "scale {value_count} added serialization bytes"
            );
        }
    }
}
