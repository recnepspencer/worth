use super::gamma::Gamma;

pub struct Alpha(pub(crate) Gamma);

pub(crate) fn make() -> super::beta::Beta {
    super::beta::Beta(None)
}
