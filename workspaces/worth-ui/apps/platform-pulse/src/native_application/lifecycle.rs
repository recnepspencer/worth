use worth_ui::facade::app::WorthUiNativeApplicationShell;

use super::PlatformPulseApplicationRuntime;
use crate::source_watch::{PlatformPulseSourceWatch, PlatformPulseSourceWatchShutdownDenial};

impl PlatformPulseApplicationRuntime {
    pub(super) fn shutdown_product(
        &mut self,
    ) -> Option<worth_ui::facade::app::WorthUiNativeApplicationShutdownReceipt> {
        self.pending_native_publications.clear();
        let theme_watch_released = if let Some(watch) = self.theme_watch.take() {
            match watch.shutdown() {
                Ok(()) => true,
                Err(denial) => {
                    self.fail(
                        super::PlatformPulseTerminalError::ThemePreference(denial),
                        self.publisher.appearance_preparation_failure(),
                    );
                    false
                }
            }
        } else {
            false
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
        let query = self
            .query_lifecycle
            .take()
            .map(crate::query_source::PlatformPulseQueryLifecycle::close);
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
        if self.terminal_error.is_some() {
            return application;
        }
        let publication = match (
            watcher,
            application.as_ref(),
            query,
            query_watcher,
            intent_watcher,
        ) {
            (
                Some(Ok(watcher)),
                Some(application),
                Some(Ok(query)),
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
            (Some(Err(PlatformPulseSourceWatchShutdownDenial::Watcher(denial))), _, _, _, _) => {
                self.publisher.filesystem_watcher_failure(&denial)
            }
            (Some(Err(PlatformPulseSourceWatchShutdownDenial::WorkerPanicked)), _, _, _, _) => {
                self.publisher.source_worker_panicked()
            }
            (_, _, Some(Err(_)), _, _) => self.publisher.query_shutdown_failure(),
            (_, _, _, _, Some(Err(_))) => self.publisher.intent_preparation_failure(),
            _ => return application,
        };
        if let Err(error) = publication {
            eprintln!("WORTH UI platform pulse shutdown evidence failed: {error:?}");
        }
        application
    }
}
