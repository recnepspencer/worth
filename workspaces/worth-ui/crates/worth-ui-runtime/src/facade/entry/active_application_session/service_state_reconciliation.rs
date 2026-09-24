use super::super::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    pub(crate) fn reconcile_service_state_after_mounted_publication(&mut self) {
        if self
            .scroll
            .as_ref()
            .is_some_and(|scroll| scroll.has_mounted_ownership())
        {
            self.reconcile_published_scroll_owners();
            // Reconciling bounds is where an owner's extent can collapse, and
            // a collapsed extent retires the target it left nowhere to reach.
            // The motion that target was driving ends here, in the same
            // reconciliation, so no frame settles content that has no room.
            self.settle_scroll_motion_without_a_target();
        }
        if let Some(scroll) = self.scroll.as_mut() {
            scroll.retire_stale_direct_successions();
        }
        self.mounted
            .reconcile_direct_scroll_evidence(self.scroll.as_ref());
        if self
            .selection
            .as_ref()
            .is_some_and(|selection| selection.has_projection_catalog_owners())
        {
            self.reconcile_published_selection_catalogs();
        }
    }

    fn reconcile_published_scroll_owners(&mut self) {
        let mut shared_owners = std::collections::BTreeMap::new();
        let mounted_instances = self
            .scroll
            .as_ref()
            .expect("mounted Scroll ownership was checked above")
            .ownership_instances()
            // Gathered before the walk: retiring an instance the mounted state
            // no longer names changes the catalog this walk reads.
            .collect::<Vec<_>>();
        for mounted_instance in mounted_instances {
            let Some(target) = self
                .mounted
                .current_mounted_identity_basis(mounted_instance)
            else {
                self.scroll
                    .as_mut()
                    .expect("mounted Scroll ownership was checked above")
                    .retire_mounted_instance(mounted_instance);
                continue;
            };
            let chain = match self
                .scroll
                .as_ref()
                .expect("mounted Scroll ownership was checked above")
                .ownership_chain(mounted_instance)
                .cloned()
            {
                Ok(chain) => chain,
                Err(_) => {
                    let incarnation =
                        crate::runtime::scroll::UiScrollOwnerIncarnation::from_mount_incarnation(
                            target.mount_incarnation(),
                        );
                    self.application.install_scroll_ownership(
                        self.scroll
                            .as_mut()
                            .expect("mounted Scroll ownership was checked above"),
                        mounted_instance,
                        incarnation,
                        &target,
                    );
                    let Ok(chain) = self
                        .scroll
                        .as_ref()
                        .expect("mounted Scroll ownership was checked above")
                        .ownership_chain(mounted_instance)
                        .cloned()
                    else {
                        continue;
                    };
                    chain
                }
            };
            let anchor = self.published_mounted_scroll_anchor(mounted_instance, &target);
            let mounted_incarnation =
                crate::runtime::scroll::UiScrollOwnerIncarnation::from_mount_incarnation(
                    target.mount_incarnation(),
                );
            let mut registrations = Vec::with_capacity(chain.owners().len());
            for (slot, owner) in chain.owners().iter().copied().enumerate() {
                let Ok(bounds) = self.scroll_bounds_for_mounted_owner(
                    owner,
                    mounted_instance,
                    target.graph_node_identity(),
                    slot,
                ) else {
                    registrations.clear();
                    break;
                };
                let incarnation = match owner {
                    crate::runtime::scroll::UiScrollOwnerIdentity::Region { .. } => {
                        let Some(incarnation) =
                            self.scroll_region_incarnation(mounted_instance, slot)
                        else {
                            registrations.clear();
                            break;
                        };
                        incarnation
                    }
                    crate::runtime::scroll::UiScrollOwnerIdentity::Surface(_)
                    | crate::runtime::scroll::UiScrollOwnerIdentity::Viewport(_) => {
                        self.scroll_owner_incarnation()
                    }
                };
                registrations.push(crate::runtime::scroll::UiScrollOwnerRegistration::new(
                    owner,
                    incarnation,
                    bounds.axes(),
                    bounds,
                    crate::runtime::scroll::UiScrollOffset::origin(),
                ));
            }
            if registrations.len() != chain.owners().len() {
                self.scroll
                    .as_mut()
                    .expect("mounted Scroll ownership was checked above")
                    .suspend_mounted_instance(mounted_instance, mounted_incarnation);
                continue;
            }
            for (owner, registration) in chain.owners().iter().copied().zip(registrations) {
                match owner {
                    crate::runtime::scroll::UiScrollOwnerIdentity::Region { .. } => {
                        // Mounted preparation already applied this offset to
                        // descendants. A newly observed anchor must not rebase
                        // the owner after the corresponding pixels are accepted.
                        self.scroll
                            .as_mut()
                            .expect("mounted Scroll ownership was checked above")
                            .reconcile_rebind(crate::runtime::scroll::UiScrollRebindRequest::new(
                                registration,
                                None,
                                crate::runtime::scroll::UiScrollAnchorPolicy::Clamp,
                            ))
                            .expect(
                                "published allocation produces a valid Scroll owner reconciliation",
                            );
                    }
                    crate::runtime::scroll::UiScrollOwnerIdentity::Surface(_)
                    | crate::runtime::scroll::UiScrollOwnerIdentity::Viewport(_) => {
                        shared_owners
                            .entry(owner)
                            .and_modify(
                                |pending: &mut crate::runtime::scroll::UiSharedScrollOwnerReconciliation| {
                                    pending.absorb(registration, mounted_instance, anchor);
                                },
                            )
                            .or_insert_with(|| {
                                crate::runtime::scroll::UiSharedScrollOwnerReconciliation::new(
                                    registration,
                                    mounted_instance,
                                    anchor,
                                )
                            });
                    }
                }
            }
        }
        for pending in shared_owners.into_values() {
            pending
                .reconcile(
                    self.scroll
                        .as_mut()
                        .expect("mounted Scroll ownership was checked above"),
                )
                .expect("published allocation produces valid shared Scroll owner reconciliation");
        }
    }

    fn reconcile_published_selection_catalogs(&mut self) {
        let families = self
            .selection
            .as_ref()
            .expect("Selection catalog ownership was checked above")
            .projection_families();
        for family in families.iter().copied() {
            let Some(slot) = family.projection_input_slot() else {
                continue;
            };
            self.mounted.refresh_selection_bindings(slot);
            let Some(worth_ui_query_binding::UiProjectionInputFactReference::Collection(
                collection,
            )) = self.mounted.current_projection_input(slot)
            else {
                self.selection
                    .as_mut()
                    .expect("Selection catalog ownership was checked above")
                    .retire_family(family);
                continue;
            };
            if collection.posture() != worth_ui_query_binding::UiProjectionInputPosture::Current {
                self.selection
                    .as_mut()
                    .expect("Selection catalog ownership was checked above")
                    .retire_family(family);
                continue;
            }
            let revision = collection.revision().observation_order();
            if !self
                .selection
                .as_ref()
                .expect("Selection catalog ownership was checked above")
                .family_requires_catalog_reconciliation(family, revision)
            {
                continue;
            }
            let Some(keys) = collection.current_application_item_keys() else {
                self.selection
                    .as_mut()
                    .expect("Selection catalog ownership was checked above")
                    .retire_family(family);
                continue;
            };
            let posture = match collection.completeness() {
                Some(worth_ui_query_binding::UiCollectionCompleteness::Complete) => {
                    crate::runtime::selection::UiSelectionCatalogPosture::Complete
                }
                Some(worth_ui_query_binding::UiCollectionCompleteness::Partial) => {
                    crate::runtime::selection::UiSelectionCatalogPosture::Partial
                }
                None => {
                    self.selection
                        .as_mut()
                        .expect("Selection catalog ownership was checked above")
                        .retire_family(family);
                    continue;
                }
            };
            self.selection
                .as_mut()
                .expect("Selection catalog ownership was checked above")
                .reconcile_projection_catalog(family, revision, &keys, posture)
                .expect("published collection retains a valid Selection catalog");
        }
    }

    fn published_mounted_scroll_anchor(
        &self,
        mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        target: &crate::mounting::UiMountedIdentityBasis,
    ) -> Option<crate::runtime::scroll::UiScrollAnchor> {
        let publication = self.mounted.current_publication()?;
        let presentation = publication
            .presentation_for_surface(target.semantic_surface_identity())
            .map(|displayed| displayed.basis())?;
        let hit_test = self.mounted.interaction_hit_test_basis(presentation).ok()?;
        let row = hit_test
            .rows()
            .iter()
            .find(|row| row.mounted_instance() == mounted_instance)?;
        let bounds = row.bounds();
        crate::runtime::scroll::UiScrollAnchor::new(
            crate::runtime::scroll::UiScrollAnchorIdentity::mounted(mounted_instance),
            presentation.binding(),
            signed_subpixels(bounds.x())?.max(0),
            signed_subpixels(bounds.y())?.max(0),
        )
    }
}

fn signed_subpixels(value: f32) -> Option<i64> {
    let scaled = f64::from(value)
        * worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
    (scaled.is_finite() && scaled >= i64::MIN as f64 && scaled <= i64::MAX as f64)
        .then(|| scaled.round() as i64)
}
