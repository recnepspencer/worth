//! Pure graph-constitution meaning. Runtime authority stays with adopting owners.
//!
//! ```rust
//! use worth_schema_graph::facade::{
//!     lower_graph_promotion_identity_basis, CarryingArtifactIdentity, DurableReferenceKind,
//!     PromotionRequest, SubelementKey,
//! };
//!
//! let request = PromotionRequest::new(
//!     DurableReferenceKind::ManualRefinement,
//!     SubelementKey::new("edge:17").unwrap(),
//! );
//! let promoted = lower_graph_promotion_identity_basis(
//!     request,
//!     CarryingArtifactIdentity::new("publication:brep:4").unwrap(),
//! );
//! assert_eq!(promoted.subelement_key().as_str(), "edge:17");
//! ```

pub mod facade;

mod promotion;
