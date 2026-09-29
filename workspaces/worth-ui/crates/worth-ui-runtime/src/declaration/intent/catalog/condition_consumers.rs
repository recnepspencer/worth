//! Application-fact and projection operability sources stay pull-only in 3a:
//! only condition sources are pushed, through this index.

use std::collections::BTreeMap;

use crate::runtime::expression::UiExpressionSlot;

/// The declarations whose operability reads each condition slot of the
/// catalog's own expression generation.
#[derive(Default)]
pub(crate) struct UiIntentConditionConsumers {
    by_slot: BTreeMap<UiExpressionSlot, Box<[Box<str>]>>,
}

impl UiIntentConditionConsumers {
    pub(super) fn index(
        declarations: &[std::sync::Arc<crate::declaration::UiCanonicalIntentDeclaration>],
    ) -> Self {
        let mut by_slot = BTreeMap::<UiExpressionSlot, Vec<Box<str>>>::new();
        for declaration in declarations {
            let identity: &str = declaration.identity().as_str();
            for condition in declaration.operability().conditions() {
                let consumers = by_slot.entry(condition.slot()).or_default();
                if consumers.last().map(AsRef::as_ref) != Some(identity) {
                    consumers.push(identity.into());
                }
            }
        }
        Self {
            by_slot: by_slot
                .into_iter()
                .map(|(slot, consumers)| (slot, consumers.into_boxed_slice()))
                .collect(),
        }
    }

    /// The declaration identities reading `slot`, each once.
    pub(crate) fn of(&self, slot: UiExpressionSlot) -> impl Iterator<Item = &str> {
        self.by_slot
            .get(&slot)
            .into_iter()
            .flat_map(|consumers| consumers.iter().map(AsRef::as_ref))
    }
}
