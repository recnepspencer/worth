//! A typed successor owns delivery custody until promotion or destruction.
use super::*;

impl<Schema, Family> Drop for TypedRequiredSuccessor<Schema, Family>
where
    Schema: ApplicationSchema,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    fn drop(&mut self) {
        // Promotion takes the demand: the receiving caller becomes its entry.
        // Every other destruction, including refused custody and end_refused,
        // releases the same delivery custody before the interest itself drops.
        if let Some(interest) = self
            .demand
            .as_ref()
            .and_then(|demand| demand.interest.as_ref())
        {
            interest.close_discarded_successor();
        }
    }
}
