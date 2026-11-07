//! Signal-owned durable source, distinct from evaluation snapshots.
pub(crate) mod authored_image;
pub use authored_image::{
    SignalAuthoredImage, SignalAuthoredImageDenial, SIGNAL_AUTHORED_IMAGE_PROTOCOL,
    SIGNAL_AUTHORED_IMAGE_VERSION,
};
