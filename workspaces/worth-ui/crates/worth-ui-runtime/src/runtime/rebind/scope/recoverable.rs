use super::resolver::{
    finish_scope, prepare_resolution, FinishScopeInput, UiAffectedScopeResolver,
};
use super::{UiAffectedScopeDenial, UiResolvedAffectedScope};
use crate::runtime::observation::UiClassifiedChange;

pub(crate) struct UiAffectedScopeRecoveryStop {
    denial: UiAffectedScopeDenial,
    change: Box<UiClassifiedChange>,
}

impl UiAffectedScopeResolver {
    pub(crate) fn resolve_recoverable(
        change: UiClassifiedChange,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        predecessor: &crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationAuthority,
    ) -> Result<UiResolvedAffectedScope, UiAffectedScopeRecoveryStop> {
        let prepared = match prepare_resolution(&change, session, predecessor) {
            Ok(prepared) => prepared,
            Err(denial) => {
                return Err(UiAffectedScopeRecoveryStop {
                    denial,
                    change: Box::new(change),
                })
            }
        };
        let (classification, facts, source_succession, theme_switch) = change.into_parts();
        finish_scope(FinishScopeInput {
            classification,
            facts,
            source_succession,
            theme_switch,
            predecessor_graph: prepared.predecessor_graph,
            candidate_generation: prepared.candidate_generation,
            candidate_graph: prepared.candidate_graph,
            lookups: prepared.accumulation.lookups,
            consumers: prepared.accumulation.consumers,
            aspects: prepared.accumulation.aspects,
        })
        .map_err(|_| unreachable!("scope finishing is infallible after borrowed validation"))
    }
}

impl UiAffectedScopeRecoveryStop {
    pub(crate) fn into_denial(self) -> UiAffectedScopeDenial {
        self.denial
    }

    pub(crate) fn into_parts(self) -> (UiAffectedScopeDenial, UiClassifiedChange) {
        (self.denial, *self.change)
    }
}
