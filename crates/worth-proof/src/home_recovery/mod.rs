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
//! An owner can transfer the same custody through the named stages:
//!
//! ```
//! use worth_proof::{AuthorityMarker, HomeAdmitted, RelationalReadmitted, StoreRecovered};
//! struct OwnerCustody(String);
//! impl AuthorityMarker for OwnerCustody {}
//! let recovered = StoreRecovered::from_owner(OwnerCustody("same custody".into()));
//! let admitted = recovered.consume(HomeAdmitted::from_owner);
//! let relational = admitted.consume(RelationalReadmitted::from_owner);
//! assert_eq!(relational.consume(|custody| custody.0), "same custody");
//! ```

mod home_admitted;
mod relational_readmitted;
mod store_recovered;

pub use home_admitted::HomeAdmitted;
pub use relational_readmitted::RelationalReadmitted;
pub use store_recovered::StoreRecovered;
