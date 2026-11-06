//! Explicit disposal of a closed Home, using Store's filesystem mutation lease.
use super::request::WorthQueryApplicationHomeLocation;
pub use worth_store_application::ApplicationHomeDiscardDenial as WorthQueryApplicationHomeDiscardDenial;

pub fn discard_home(
    location: WorthQueryApplicationHomeLocation,
) -> Result<(), WorthQueryApplicationHomeDiscardDenial> {
    match location {
        WorthQueryApplicationHomeLocation::MemoryOnly => Ok(()),
        WorthQueryApplicationHomeLocation::Directory(path) => {
            worth_store_application::discard_home(path)
        }
    }
}
