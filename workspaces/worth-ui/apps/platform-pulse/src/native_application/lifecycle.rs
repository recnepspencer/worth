use worth_ui::facade::app::WorthUiNativeApplicationShell;

use super::PlatformPulseApplicationRuntime;
use crate::source_watch::{PlatformPulseSourceWatch, PlatformPulseSourceWatchShutdownDenial};

impl PlatformPulseApplicationRuntime {
    pub(super) fn close_native_runtime(
        mut self: Box<Self>,
        application: WorthUiNativeApplicationShell,
    ) -> Result<
        worth_ui_native_platform::UiNativeApplicationRuntimeClosed,
        worth_ui_native_platform::UiNativeApplicationRuntimeCloseIncomplete,
    > {
        self.shell = Some(application);
        self.external_close_requested = true;
        self.advance_closing_query_turn();
        let shutdown = match self.shutdown_product() {
            Some(shutdown) => shutdown,
            None => {
                let application = self.take_runtime_shell();
                return Err(
                    worth_ui_native_platform::UiNativeApplicationRuntimeCloseIncomplete::retain(
                        self,
                        application,
                    ),
                );
            }
        };
        Ok(
            worth_ui_native_platform::UiNativeApplicationRuntimeClosed::from_application_shutdown(
                shutdown,
            ),
        )
    }

    pub(super) fn shutdown_product(
        &mut self,
    ) -> Option<worth_ui::facade::app::WorthUiNativeApplicationShutdownReceipt> {
        // Request, cancellation and presentation obligations stay in the runtime
        // until their actual existing owners have settled them.
        if !self.query_close_ready() {
            return None;
        }
        // Query refusal preserves the whole lifecycle before any shell/watch teardown,
        // including when the product is already terminal.
        let query = match close_query_for_shutdown(&mut self.query_lifecycle) {
            Ok(query) => query,
            Err(_) => return None,
        };
        self.pending_native_publications.clear();
        let theme_watch_released = match self.theme_watch.take() {
            Some(watch) => match watch.shutdown() {
                Ok(()) => true,
                Err(denial) => {
                    self.fail(
                        super::PlatformPulseTerminalError::ThemePreference(denial),
                        self.publisher.appearance_preparation_failure(),
                    );
                    false
                }
            },
            _ => false,
        };
        let visual_shutdown = self
            .shell
            .as_mut()
            .map(|shell| self.visual_identity.shutdown_quiescent(shell));
        if let Some(Err(denial)) = visual_shutdown {
            self.fail_visual_identity(denial);
        }
        let watcher = self
            .source_watch
            .take()
            .map(PlatformPulseSourceWatch::shutdown);
        let query_watcher = self
            .query_watch
            .take()
            .map(crate::query_source::PlatformPulseExternalValueWatch::shutdown);
        let intent_watcher = self
            .intent_watch
            .take()
            .map(worth_ui_platform_pulse::intent::PlatformPulseIntentInputWatch::shutdown);
        self.intent_gate.take();
        self.intent_action_owner.take();
        let application = self
            .shell
            .take()
            .map(WorthUiNativeApplicationShell::shutdown);
        if self.terminal.is_stopped() {
            return application;
        }
        let publication = match (watcher, application.as_ref(), query_watcher, intent_watcher) {
            (
                Some(Ok(watcher)),
                Some(application),
                Some(Ok(query_watcher)),
                Some(Ok(intent_watcher)),
            ) => self.publisher.shutdown(
                &watcher,
                query,
                query_watcher,
                intent_watcher,
                theme_watch_released,
                application,
            ),
            (Some(Err(PlatformPulseSourceWatchShutdownDenial::Watcher(denial))), _, _, _) => {
                self.publisher.filesystem_watcher_failure(&denial)
            }
            (Some(Err(PlatformPulseSourceWatchShutdownDenial::WorkerPanicked)), _, _, _) => {
                self.publisher.source_worker_panicked()
            }
            (_, _, _, Some(Err(_))) => self.publisher.intent_preparation_failure(),
            _ => return application,
        };
        if let Err(error) = publication {
            eprintln!("WORTH UI platform pulse shutdown evidence failed: {error:?}");
        }
        application
    }
}

/// The production close gate restores the whole refused Query owner. A refusal
/// leaves shell, subscriptions, watches and terminal posture with the native runtime.
fn close_query_for_shutdown(
    owner: &mut Option<crate::query_source::PlatformPulseQueryLifecycle>,
) -> Result<
    crate::query_source::PlatformPulseQueryShutdownReceipt,
    crate::query_source::PlatformPulseQueryLifecycleDenial,
> {
    let lifecycle = owner
        .take()
        .ok_or(crate::query_source::PlatformPulseQueryLifecycleDenial::OwnerNotLive)?;
    match lifecycle.close() {
        Ok(closed) => Ok(closed),
        Err((lifecycle, denial)) => {
            *owner = Some(*lifecycle);
            Err(denial)
        }
    }
}

#[cfg(test)]
mod tests;
