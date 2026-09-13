//! Public consumption flows through [`facade`] only.
//!
//! ```rust
//! use worth_schema_core::facade::{Identity, IdentityName, Tolerance, Unit};
//!
//! let identity_name = IdentityName::new("wall_panel").unwrap();
//! let identity = Identity::named(identity_name);
//! let tolerance = Tolerance::microns(50).unwrap();
//! let unit = Unit::millimeters();
//!
//! assert!(matches!(identity, Identity::Named(_)));
//! assert_eq!(tolerance.as_microns(), 50);
//! assert_eq!(unit.symbol(), "mm");
//! ```
//!
//! ```compile_fail
//! use worth_schema_core::identity::Identity;
//! ```

pub mod facade;

mod identity;
mod identity_name;
mod naming;
mod tolerance;
mod units;
