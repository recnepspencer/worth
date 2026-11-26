//! The installed fields of one entity, in canonical (aspect, field) order.

use worth_foundational::facade::AspectFieldLocator;

use super::WorthQueryPrimaryGraphLayout;

/// One installed field of an entity: its declared aspect and field names and
/// its physical locator.
pub struct WorthQueryEntityFieldLayout<'layout> {
    pub aspect: &'layout str,
    pub field: &'layout str,
    pub locator: &'layout AspectFieldLocator,
}

impl WorthQueryPrimaryGraphLayout {
    /// Every installed field of `entity`. Bounded by the entity's declared
    /// fields; a range read of the field table, no scan of other entities.
    pub fn entity_fields<'layout>(
        &'layout self,
        entity: &'layout str,
    ) -> impl Iterator<Item = WorthQueryEntityFieldLayout<'layout>> + 'layout {
        self.fields
            .range((entity.to_owned(), String::new(), String::new())..)
            .take_while(move |((owner, _, _), _)| owner == entity)
            .map(|((_, aspect, field), layout)| WorthQueryEntityFieldLayout {
                aspect,
                field,
                locator: &layout.locator,
            })
    }
}
