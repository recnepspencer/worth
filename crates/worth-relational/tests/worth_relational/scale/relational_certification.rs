#[allow(unused_imports)]
pub(crate) use crate::fast::relational_certification::*;

#[path = "relational_certification/root/cost/scale_axes.rs"]
pub(crate) mod root_cost_scale_axes;

#[path = "relational_certification/root/sharing/fork.rs"]
pub(crate) mod root_fork_sharing;

#[path = "relational_certification/invariants/admission/scale_profile.rs"]
pub(crate) mod scale_invariant_admission;
