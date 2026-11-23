use super::fork_overlay::{ForkSubscriptions, ReverseSubscriptionStorage};
use super::ReverseSubscriptionIndex;
use crate::data::retained_storage::SignalConditionalRetentionReservation;

impl ReverseSubscriptionIndex {
    pub(crate) fn fork_persistent(&mut self) -> Self {
        self.fork_with_resources(None)
    }
    pub(crate) fn fork_reserved(
        &mut self,
        resources: &mut SignalConditionalRetentionReservation,
    ) -> Self {
        self.fork_with_resources(Some(resources))
    }
    fn fork_with_resources(
        &mut self,
        resources: Option<&mut SignalConditionalRetentionReservation>,
    ) -> Self {
        if let ReverseSubscriptionStorage::Exclusive(flat) = &mut self.storage {
            let custody = resources.map(|resources| {
                let growth = ForkSubscriptions::conversion_charge()
                    .expect("fixed reverse-index conversion fits");
                resources
                    .split_embedded(growth)
                    .expect("complete reverse conversion was reserved")
            });
            self.storage = ReverseSubscriptionStorage::ForkShared(ForkSubscriptions::convert(
                std::mem::take(flat),
                custody,
            ));
        }
        self.fork_storage_identity_impl()
    }

    #[cfg(test)]
    pub(crate) fn fork_storage_identity(&self) -> Self {
        self.fork_storage_identity_impl()
    }

    fn fork_storage_identity_impl(&self) -> Self {
        let storage = match &self.storage {
            ReverseSubscriptionStorage::ForkShared(overlay) => {
                ReverseSubscriptionStorage::ForkShared(overlay.clone())
            }
            ReverseSubscriptionStorage::Exclusive(_) => {
                unreachable!("persistent fork must install shared storage")
            }
        };
        Self {
            storage,
            valid: self.valid,
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_storage_with(&self, other: &Self) -> bool {
        matches!(
            (&self.storage, &other.storage),
            (
                ReverseSubscriptionStorage::ForkShared(left),
                ReverseSubscriptionStorage::ForkShared(right),
            ) if std::sync::Arc::ptr_eq(&left.maps().base, &right.maps().base)
                && left.maps().bucket_changes.ptr_eq(&right.maps().bucket_changes)
                && left.maps().consumer_changes.ptr_eq(&right.maps().consumer_changes)
        )
    }
}
