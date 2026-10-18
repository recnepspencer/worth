//! Installs the existing five readiness owners and retains Query's actual close wake port.
use super::PlatformPulseApplicationRuntime;
impl PlatformPulseApplicationRuntime {
    pub(super) fn install_native_readiness(
        &mut self,
        readiness: Box<[worth_ui_native_platform::UiNativeApplicationReadinessPort]>,
    ) -> Result<worth_ui_native_platform::UiNativeApplicationReadinessPort, ()> {
        let readiness: [worth_ui_native_platform::UiNativeApplicationReadinessPort; 5] =
            readiness.into_vec().try_into().map_err(|_| ())?;
        let [startup, source, query, product_input, visual] = readiness;
        self.source_watch.as_ref().ok_or(())?.install_readiness(
            worth_ui_platform_pulse::PlatformPulseApplicationReadinessSignal::from_native(source),
        );
        self.query_readiness = Some(query.clone());
        self.query_watch.as_ref().ok_or(())?.install_readiness(
            worth_ui_platform_pulse::PlatformPulseApplicationReadinessSignal::from_native(query),
        );
        // Both application-owned inputs are drained on the same product turn.
        // Coalescing their wake carries no Intent or theme publication authority.
        let product_input =
            worth_ui_platform_pulse::PlatformPulseApplicationReadinessSignal::from_native(
                product_input,
            );
        self.intent_watch
            .as_ref()
            .ok_or(())?
            .install_readiness(product_input.clone());
        self.theme_watch
            .as_ref()
            .ok_or(())?
            .install_readiness(product_input);
        self.visual_identity.install_readiness(
            worth_ui_platform_pulse::PlatformPulseApplicationReadinessSignal::from_native(visual),
        );
        Ok(startup)
    }
}
