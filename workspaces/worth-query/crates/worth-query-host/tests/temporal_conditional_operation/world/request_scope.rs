//! request scope.
use super::*;

pub fn request_scope() -> admission::authenticated_principal::WorthQueryRequestScope {
    let cancellation = admission::authenticated_principal::WorthQueryCancellationSource::new();
    admission::authenticated_principal::WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    )
}
