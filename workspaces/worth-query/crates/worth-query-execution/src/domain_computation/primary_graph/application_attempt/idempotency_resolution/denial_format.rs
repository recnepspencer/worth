use super::WorthQueryApplicationIdempotencyResolutionDenial;

impl std::fmt::Display for WorthQueryApplicationIdempotencyResolutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application idempotency resolution denied: {:?}",
            self.kind()
        )
    }
}

impl std::error::Error for WorthQueryApplicationIdempotencyResolutionDenial {}
