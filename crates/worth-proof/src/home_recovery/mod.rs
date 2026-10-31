//! Named recovery stages carrying the owning runtime's actual custody.
//!
//! Physical completion cannot stand in for descriptor admission:
//!
//! ```compile_fail,E0308
//! use worth_proof::{AuthorityMarker, HomeAdmitted, StoreRecovered};
//! struct OwnerCustody;
//! impl AuthorityMarker for OwnerCustody {}
//! fn readmit(_: HomeAdmitted<OwnerCustody>) {}
//! readmit(StoreRecovered::from_owner(OwnerCustody));
//! ```
//!
//! Custody advances once:
//!
//! ```compile_fail,E0382
//! use worth_proof::{AuthorityMarker, StoreRecovered};
//! struct OwnerCustody;
//! impl AuthorityMarker for OwnerCustody {}
//! let stage = StoreRecovered::from_owner(OwnerCustody);
//! stage.consume(|_| ());
//! stage.consume(|_| ());
//! ```
//!
//! Each actual transition mints a distinct owner-sealed carrier while moving
//! the same underlying custody. Rewrapping an extracted predecessor does not
//! satisfy the next concrete port:
//!
//! ```compile_fail,E0308
//! use worth_proof::{AuthorityMarker, HomeAdmitted, StoreRecovered};
//! struct RecoveredCustody;
//! impl AuthorityMarker for RecoveredCustody {}
//! struct AdmittedCustody;
//! impl AuthorityMarker for AdmittedCustody {}
//! fn readmit(_: HomeAdmitted<AdmittedCustody>) {}
//! let recovered = StoreRecovered::from_owner(RecoveredCustody);
//! let relabeled = recovered.consume(HomeAdmitted::from_owner);
//! readmit(relabeled);
//! ```
//!
//! The owner transfer preserves the underlying value:
//!
//! ```
//! mod owner {
//!     use worth_proof::{AuthorityMarker, HomeAdmitted, StoreRecovered};
//!     pub struct RecoveredCustody { lease: String }
//!     pub struct AdmittedCustody { lease: String }
//!     impl AuthorityMarker for RecoveredCustody {}
//!     impl AuthorityMarker for AdmittedCustody {}
//!     pub fn recover() -> StoreRecovered<RecoveredCustody> {
//!         StoreRecovered::from_owner(RecoveredCustody { lease: "same custody".into() })
//!     }
//!     pub fn admit(stage: StoreRecovered<RecoveredCustody>) -> HomeAdmitted<AdmittedCustody> {
//!         stage.consume(|previous| HomeAdmitted::from_owner(AdmittedCustody { lease: previous.lease }))
//!     }
//!     pub fn finish(stage: HomeAdmitted<AdmittedCustody>) -> String {
//!         stage.consume(|admitted| admitted.lease)
//!     }
//! }
//! assert_eq!(owner::finish(owner::admit(owner::recover())), "same custody");
//! ```

mod home_admitted;
mod world_reconciled;
pub use world_reconciled::WorldReconciled;
mod relational_readmitted;
mod store_recovered;

pub use home_admitted::HomeAdmitted;
pub use relational_readmitted::RelationalReadmitted;
pub use store_recovered::StoreRecovered;
