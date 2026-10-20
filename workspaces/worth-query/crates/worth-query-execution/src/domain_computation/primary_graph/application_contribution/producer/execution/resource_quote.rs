//! Erasing a source does not erase the reason its concrete quote failed.
use super::super::{WorthQueryProducerDemandResources, WorthQueryProducerResourceQuoteDenial};
use std::any::Any;

pub(super) fn from_source<T: 'static>(
    source: &dyn Any,
    quote: impl FnOnce(
        &T,
    ) -> Result<
        WorthQueryProducerDemandResources,
        WorthQueryProducerResourceQuoteDenial,
    >,
) -> Result<Option<WorthQueryProducerDemandResources>, WorthQueryProducerResourceQuoteDenial> {
    source.downcast_ref::<T>().map(quote).transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erased_source_mismatch_and_checked_quote_failure_remain_distinct() {
        let wrong = from_source::<usize>(&"not the source", |_| {
            panic!("a foreign source must not reach the provider quote")
        });
        assert_eq!(wrong, Ok(None));
        let quote = |count: &usize| {
            let bytes = count
                .checked_mul(std::mem::size_of::<u64>())
                .ok_or(WorthQueryProducerResourceQuoteDenial::SizeOverflow)?;
            Ok(WorthQueryProducerDemandResources::new(None, bytes))
        };
        assert_eq!(
            from_source(&usize::MAX, quote),
            Err(WorthQueryProducerResourceQuoteDenial::SizeOverflow),
        );
        assert_eq!(
            from_source(&2usize, quote),
            Ok(Some(WorthQueryProducerDemandResources::new(None, 16))),
        );
        for cause in [
            WorthQueryProducerResourceQuoteDenial::SourceNotEncodable,
            WorthQueryProducerResourceQuoteDenial::ScratchReservationRefused,
        ] {
            assert_eq!(from_source::<usize>(&2usize, |_| Err(cause)), Err(cause),);
        }
    }
}
