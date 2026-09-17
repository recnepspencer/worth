use worth_signal::facade::runtime::{
    ObservationListener, ObservationNotice, ObservationReadContext,
};

use super::SharedStore;
use crate::runtime::web_callbacks::{self, ObservationCallbackToken};

pub(super) struct WasmWatchListener {
    pub(super) callback_scope_id: u64,
    pub(super) callback_token: ObservationCallbackToken,
    pub(super) signal_id: String,
}

impl ObservationListener<(), (), (), SharedStore, ()> for WasmWatchListener {
    fn on_observation(
        &self,
        ctx: ObservationReadContext<'_, (), (), (), SharedStore, ()>,
        notice: &ObservationNotice<'_>,
    ) {
        web_callbacks::invoke_watch(
            self.callback_scope_id,
            self.callback_token,
            web_callbacks::notice_from_runtime(&self.signal_id, ctx, notice),
        );
    }
}

pub(super) struct WasmEffectListener {
    pub(super) callback_scope_id: u64,
    pub(super) callback_token: ObservationCallbackToken,
}

impl ObservationListener<(), (), (), SharedStore, ()> for WasmEffectListener {
    fn on_observation(
        &self,
        _ctx: ObservationReadContext<'_, (), (), (), SharedStore, ()>,
        _notice: &ObservationNotice<'_>,
    ) {
        web_callbacks::invoke_effect(self.callback_scope_id, self.callback_token);
    }
}
