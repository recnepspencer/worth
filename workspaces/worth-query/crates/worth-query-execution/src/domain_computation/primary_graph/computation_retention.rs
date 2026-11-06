//! What one partitioned computation retains between the runs of its producer,
//! and why a record retains nothing.
//!
//! The basis a retained run is the same run for (its owner's installation,
//! the producer edition and the input digest), the reason a record holds no
//! reusable state, and the cause a run recomputed every partition. Nothing
//! here names a module of the primary graph: the producer registry admits an
//! edition through `ProducerEditionAdmission`, and the partitioned computation
//! decides reuse over these values.
pub(crate) mod full_cause;
pub(crate) mod prior_absence;
pub(crate) mod producer_edition;
pub(crate) mod retained_basis;
