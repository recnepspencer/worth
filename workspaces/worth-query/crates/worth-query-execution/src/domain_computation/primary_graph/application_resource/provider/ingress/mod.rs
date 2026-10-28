//! Finite pre-admitted ingress; unverified frames are not Native queue items.
mod backing;
mod sealed_frame;
mod writer;
pub(in crate::domain_computation::primary_graph::application_resource) use backing::IngressBacking;
pub(in crate::domain_computation::primary_graph::application_resource) use sealed_frame::SealedIngressFrame;
pub use writer::{WorthQueryResourceIngressDenial, WorthQueryResourceIngressWriter};
