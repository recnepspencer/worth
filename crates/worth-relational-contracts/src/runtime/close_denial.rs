//! Descriptive cause returned by the actual owner-close admission.
use crate::change_source::read_custody::RelationalReadCustodyHolders;
use crate::publication::close_custody::RelationalPublicationCustodyHolders;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RelationalRuntimeCloseDenial {
    NotOwner,
    OperationsInFlight,
    PublicationCustodyRetained(RelationalPublicationCustodyHolders),
    ReadCustodyRetained(RelationalReadCustodyHolders),
}
