mod completion;
mod seed;

pub(in crate::mounting::projection) use completion::{complete_hit_test, rebind_hit_tests};
pub(in crate::mounting) use completion::{reattribute_hit_test, reattribute_hit_test_with_probes};
pub(in crate::mounting::projection) use seed::{lower_hit_test_seed, UiMountedHitTestSeed};
