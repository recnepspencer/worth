//! A decision seals complete-content comparison; candidate construction never rereads it.
pub enum CreationDisposition {
    Create,
    Reuse,
}
