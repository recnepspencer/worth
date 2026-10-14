//! Listener shutdown refuses before disconnecting live native mutation custody.
use super::*;
impl BankHttpServer {
    pub async fn shutdown(mut self) -> Result<BankHttpServerClose, BankHttpServerCloseFailure> {
        if let Some(mutations) = self.mutations.as_ref() {
            match mutations.quiesce().await {
                Ok(0) => {}
                Ok(_) => {
                    return Err(BankHttpServerCloseFailure::retained(
                        io::Error::other("native mutation custody remains"),
                        self,
                    ))
                }
                Err(()) => {
                    return Err(BankHttpServerCloseFailure::retained(
                        io::Error::other("mutation custody inspection unavailable"),
                        self,
                    ))
                }
            }
        }
        self.live_shutdown.send_replace(true);
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        let mut first_error = None;
        if let Some(task) = self.server_task.take() {
            retain_first_error(
                &mut first_error,
                task.await
                    .map_err(io::Error::other)
                    .and_then(|result| result),
            );
        }
        for task in [
            self.dispatcher_task.take(),
            self.continuation_task.take(),
            self.recovery_task.take(),
            self.elevation_task.take(),
        ]
        .into_iter()
        .flatten()
        {
            retain_first_error(&mut first_error, task.await.map_err(io::Error::other));
        }
        if let Some(task) = self.rail_maintenance_task.take() {
            retain_first_error(
                &mut first_error,
                task.await
                    .map_err(io::Error::other)
                    .and_then(|result| result),
            );
        }
        if let Some(thread) = self.live_thread.take() {
            retain_first_error(
                &mut first_error,
                tokio::task::spawn_blocking(move || {
                    thread
                        .join()
                        .map_err(|_| io::Error::other("Bank HTTP live executor panicked"))
                        .and_then(|result| result)
                })
                .await
                .map_err(io::Error::other)
                .and_then(|result| result),
            );
        }
        let rail_completion = match self.rail_completion.take() {
            Some(route) => Some(
                inbound_completion::BankRailCompletionContinuation::new(
                    route,
                    self.rail_maintenance_deadline,
                )
                .close()
                .await,
            ),
            None => None,
        };
        if let Some(mutations) = self.mutations.take() {
            retain_first_error(
                &mut first_error,
                mutations
                    .join_closed()
                    .await
                    .map_err(|_| io::Error::other("mutation worker join failed")),
            );
        }
        let close = BankHttpServerClose::new(rail_completion);
        match first_error {
            Some(error) => Err(BankHttpServerCloseFailure::new(error, close)),
            None => Ok(close),
        }
    }
}

fn retain_first_error(first: &mut Option<io::Error>, result: io::Result<()>) {
    if let Err(error) = result {
        if first.is_none() {
            *first = Some(error);
        }
    }
}
