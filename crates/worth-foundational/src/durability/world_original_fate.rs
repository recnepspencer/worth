//! Original World publication fate, independent of its runtime receiver.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldOriginalFate {
    Open,
    Continued,
    Released,
    Published,
}
