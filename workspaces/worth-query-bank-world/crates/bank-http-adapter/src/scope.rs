use std::future::Future;

use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope, WorthQueryRequestTimeRemaining,
};

pub(crate) async fn await_in_scope<F>(
    scope: &WorthQueryRequestScope,
    future: F,
) -> Result<F::Output, WorthQueryRequestInterruption>
where
    F: Future,
{
    if let Some(interruption) = scope.interruption() {
        return Err(interruption);
    }
    let deadline = async {
        match scope.remaining() {
            WorthQueryRequestTimeRemaining::Unbounded => std::future::pending::<()>().await,
            WorthQueryRequestTimeRemaining::Remaining(duration) => {
                tokio::time::sleep(duration).await
            }
            WorthQueryRequestTimeRemaining::Elapsed => {}
        }
    };
    tokio::select! {
        biased;
        _ = scope.cancellation().cancelled() => {
            Err(WorthQueryRequestInterruption::Cancelled)
        }
        _ = deadline => {
            Err(WorthQueryRequestInterruption::DeadlineExceeded)
        }
        output = future => Ok(output),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource;

    #[tokio::test]
    async fn unbounded_scope_allows_authentication_to_finish() {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(None, cancellation.token());
        assert_eq!(
            await_in_scope(&scope, async { "authenticated" }).await,
            Ok("authenticated")
        );
    }

    #[tokio::test]
    async fn elapsed_scope_refuses_before_polling_authentication() {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(Instant::now() - Duration::from_secs(1)),
            cancellation.token(),
        );
        assert_eq!(
            await_in_scope(&scope, async {
                panic!("elapsed authentication must not start");
            })
            .await,
            Err(WorthQueryRequestInterruption::DeadlineExceeded)
        );
    }
}
