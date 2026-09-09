use std::rc::Rc;

impl super::WorthUiPlanRegionStore {
    pub(crate) fn mounted_projection_plan_index(&self, provenance: u64) -> Result<Option<u32>, ()> {
        let Some(entries) = self.mounted_projection_index.get(&provenance) else {
            return Ok(None);
        };
        let mut entries = entries.iter().copied();
        let first = entries.next();
        if entries.next().is_some() {
            return Err(());
        }
        Ok(first)
    }

    pub(crate) fn mounted_projection_ordinary_meaning(
        &self,
        plan_index: u32,
    ) -> Option<Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>>
    {
        self.executable_for_stable_slot(u64::from(plan_index))
            .and_then(|executable| executable.ordinary_meaning_reference())
    }

    pub(crate) fn mounted_projection_ordinary_meaning_with_identity(
        &self,
        plan_index: u32,
    ) -> Option<(
        String,
        Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>,
    )> {
        let slot = u64::from(plan_index);
        let identity = self
            .handle_for_stable_slot(slot)?
            .region_identity()
            .exact_basis()
            .to_owned();
        let meaning = self
            .executable_for_stable_slot(slot)?
            .ordinary_meaning_reference()?;
        Some((identity, meaning))
    }

    pub(crate) fn mounted_projection_ordinary_meaning_for_identity(
        &self,
        identity: &str,
    ) -> Option<(
        u32,
        Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>,
    )> {
        let identity = super::super::WorthUiPlanRegionIdentity::from_exact_basis(identity);
        let plan_index = u32::try_from(self.handle_for(&identity)?.stable_slot()).ok()?;
        let meaning = self
            .executable_for(&identity)?
            .ordinary_meaning_reference()?;
        Some((plan_index, meaning))
    }

    pub(crate) fn mounted_projection_theme_token(
        &self,
        token_id: &crate::capability::ThemeTokenId,
    ) -> Result<
        Option<(
            u32,
            Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>,
        )>,
        (),
    > {
        let Some(indexes) = self.mounted_theme_token_index.get(token_id) else {
            return Ok(None);
        };
        let mut indexes = indexes.iter().copied();
        let Some(plan_index) = indexes.next() else {
            return Ok(None);
        };
        if indexes.next().is_some() {
            return Err(());
        }
        self.mounted_projection_ordinary_meaning(plan_index)
            .map(|meaning| (plan_index, meaning))
            .ok_or(())
            .map(Some)
    }

    pub(super) fn insert_mounted_projection_record(
        &mut self,
        record: &super::WorthUiPlanRegionRecord,
    ) {
        self.insert_mounted_theme_token_record(record);
        let Some((provenance, plan_index)) = mounted_projection_entry(record) else {
            return;
        };
        let mut entries = self
            .mounted_projection_index
            .get(&provenance)
            .cloned()
            .unwrap_or_default();
        entries.insert(plan_index);
        self.mounted_projection_index.insert(provenance, entries);
    }

    pub(super) fn remove_mounted_projection_record(
        &mut self,
        record: &super::WorthUiPlanRegionRecord,
    ) {
        if let Some((provenance, plan_index)) = mounted_projection_entry(record) {
            if let Some(mut entries) = self.mounted_projection_index.get(&provenance).cloned() {
                entries.remove_with_work(&plan_index);
                if entries.is_empty() {
                    self.mounted_projection_index.remove(&provenance);
                } else {
                    self.mounted_projection_index.insert(provenance, entries);
                }
            }
        }
        self.remove_mounted_theme_token_record(record);
    }

    fn insert_mounted_theme_token_record(&mut self, record: &super::WorthUiPlanRegionRecord) {
        let Some((token_id, plan_index)) = mounted_theme_token_entry(record) else {
            return;
        };
        let mut indexes = self
            .mounted_theme_token_index
            .get(&token_id)
            .cloned()
            .unwrap_or_default();
        indexes.insert(plan_index);
        self.mounted_theme_token_index.insert(token_id, indexes);
    }

    fn remove_mounted_theme_token_record(&mut self, record: &super::WorthUiPlanRegionRecord) {
        let Some((token_id, plan_index)) = mounted_theme_token_entry(record) else {
            return;
        };
        let Some(mut indexes) = self.mounted_theme_token_index.get(&token_id).cloned() else {
            return;
        };
        indexes.remove_with_work(&plan_index);
        if indexes.is_empty() {
            self.mounted_theme_token_index.remove(&token_id);
        } else {
            self.mounted_theme_token_index.insert(token_id, indexes);
        }
    }
}

fn mounted_projection_entry(record: &super::WorthUiPlanRegionRecord) -> Option<(u64, u32)> {
    let input = record.schema.input();
    if input.owner_identity_basis().is_some() {
        return None;
    }
    Some((
        input.mounted_projection_provenance_digest()?,
        u32::try_from(record.handle.stable_slot()).ok()?,
    ))
}

fn mounted_theme_token_entry(
    record: &super::WorthUiPlanRegionRecord,
) -> Option<(crate::capability::ThemeTokenId, u32)> {
    let meaning = record.executable.ordinary_meaning_reference()?;
    let crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning::Token(token) =
        meaning.as_ref()
    else {
        return None;
    };
    Some((
        crate::capability::ThemeTokenId::new(token.token_id()).ok()?,
        u32::try_from(record.handle.stable_slot()).ok()?,
    ))
}
