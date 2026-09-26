use super::{WorthUiActiveApplicationSession, WorthUiPreparedApplicationActivation};

pub(super) struct UiPreparedScrollReplacement(Option<crate::runtime::scroll::UiScrollRuntimeState>);

impl UiPreparedScrollReplacement {
    pub(super) const fn empty() -> Self {
        Self(None)
    }

    pub(super) fn state_mut(
        &mut self,
    ) -> Option<&mut crate::runtime::scroll::UiScrollRuntimeState> {
        self.0.as_mut()
    }

    pub(super) fn into_state(
        self,
    ) -> crate::runtime::UiRuntimeServiceInstallation<crate::runtime::scroll::UiScrollRuntimeState>
    {
        crate::runtime::UiRuntimeServiceInstallation::from_optional(self.0)
    }
}

impl WorthUiActiveApplicationSession {
    pub(super) fn prepare_scroll_replacement_state(
        &self,
        application: &WorthUiPreparedApplicationActivation,
    ) -> UiPreparedScrollReplacement {
        let Some(policy) = application.candidate_service_policy_plan().scroll() else {
            return UiPreparedScrollReplacement(None);
        };
        let mut scroll = self.scroll.as_ref().cloned().unwrap_or_else(|| {
            crate::runtime::scroll::UiScrollRuntimeState::new_session_restore_candidate_with_policy(
                policy,
            )
        });
        scroll.apply_policy(policy);
        UiPreparedScrollReplacement(Some(scroll))
    }

    pub(super) fn prepare_scroll_replacement(
        &self,
        application: &WorthUiPreparedApplicationActivation,
        successor: &crate::mounting::UiMountedGraphReplacementSuccessor,
        publication: Option<&crate::mounting::UiMountedFramePublicationReceipt>,
        mut prepared: UiPreparedScrollReplacement,
    ) -> UiPreparedScrollReplacement {
        let Some(scroll) = prepared.state_mut() else {
            return prepared;
        };
        let predecessor = self.mounted.view();
        let successor_view = successor.identity_view();
        for prior in predecessor.mounted_instances() {
            let retained_exactly = successor_view
                .mounted_instances()
                .iter()
                .any(|next| next.identity() == prior.identity() && next.basis() == prior.basis());
            if !retained_exactly {
                scroll.retire_mounted_instance(prior.identity());
            }
        }
        prepare_successor_ownership(
            scroll,
            application,
            successor,
            publication,
            &self.mounted,
            self.scroll_owner_incarnation(),
        );
        prepared
    }
}

fn prepare_successor_ownership(
    scroll: &mut crate::runtime::scroll::UiScrollRuntimeState,
    application: &WorthUiPreparedApplicationActivation,
    successor: &crate::mounting::UiMountedGraphReplacementSuccessor,
    publication: Option<&crate::mounting::UiMountedFramePublicationReceipt>,
    mounted: &crate::mounting::WorthUiMountedSessionState,
    surface_incarnation: crate::runtime::scroll::UiScrollOwnerIncarnation,
) {
    let mut shared_owners = std::collections::BTreeMap::new();
    let plan =
        crate::mounting::UiMountedPlanProjectionSource::Executed(application.candidate_plan());
    let catalog = application.candidate_allocation_catalog();
    for next in successor.identity_view().mounted_instances() {
        let mounted_incarnation =
            crate::runtime::scroll::UiScrollOwnerIncarnation::from_mount_incarnation(
                next.mount_incarnation(),
            );
        scroll.resolve_and_install_ownership(
            next.identity(),
            mounted_incarnation,
            application.candidate_graph(),
            plan,
            next.graph_node_identity(),
            next.basis().semantic_surface_identity(),
            next.basis().repeated_instance_basis().identity_digest(),
        );
        let Ok(chain) = scroll.ownership_chain(next.identity()).cloned() else {
            continue;
        };
        let anchor = publication
            .and_then(|receipt| published_anchor(receipt, mounted, next.identity(), next.basis()));
        let mut registrations = Vec::with_capacity(chain.owners().len());
        for (slot, owner) in chain.owners().iter().copied().enumerate() {
            let (bounds, incarnation) = match owner {
                crate::runtime::scroll::UiScrollOwnerIdentity::Region { .. } => {
                    let Some((mounted_owner, region)) = successor.scroll_region_geometry(
                        owner.semantic_surface(),
                        next.identity(),
                        slot,
                    ) else {
                        registrations.clear();
                        break;
                    };
                    let Some(bounds) = region.in_layout_space().bounds() else {
                        registrations.clear();
                        break;
                    };
                    let Some(basis) = successor
                        .layout_validation_identity()
                        .projection_instance(mounted_owner)
                    else {
                        registrations.clear();
                        break;
                    };
                    (
                        bounds,
                        crate::runtime::scroll::UiScrollOwnerIncarnation::from_mount_incarnation(
                            basis.mount_incarnation(),
                        ),
                    )
                }
                crate::runtime::scroll::UiScrollOwnerIdentity::Surface(_)
                | crate::runtime::scroll::UiScrollOwnerIdentity::Viewport(_) => {
                    let Some(bounds) = application.candidate_scroll_bounds(
                        owner,
                        next.graph_node_identity(),
                        &catalog,
                    ) else {
                        registrations.clear();
                        break;
                    };
                    (bounds, surface_incarnation)
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
            scroll.suspend_mounted_instance(next.identity(), mounted_incarnation);
            continue;
        }
        for (owner, registration) in chain.owners().iter().copied().zip(registrations) {
            match owner {
                crate::runtime::scroll::UiScrollOwnerIdentity::Region { .. } => {
                    scroll
                        .reconcile_rebind(crate::runtime::scroll::UiScrollRebindRequest::new(
                            registration,
                            None,
                            crate::runtime::scroll::UiScrollAnchorPolicy::Clamp,
                        ))
                        .expect("prepared replacement retains valid Scroll geometry");
                }
                crate::runtime::scroll::UiScrollOwnerIdentity::Surface(_)
                | crate::runtime::scroll::UiScrollOwnerIdentity::Viewport(_) => {
                    shared_owners
                        .entry(owner)
                        .and_modify(
                            |pending: &mut crate::runtime::scroll::UiSharedScrollOwnerReconciliation| {
                                pending.absorb(registration, next.identity(), anchor);
                            },
                        )
                        .or_insert_with(|| {
                            crate::runtime::scroll::UiSharedScrollOwnerReconciliation::new(
                                registration,
                                next.identity(),
                                anchor,
                            )
                        });
                }
            }
        }
    }
    for pending in shared_owners.into_values() {
        pending
            .reconcile(scroll)
            .expect("prepared replacement retains valid shared Scroll geometry");
    }
}

fn published_anchor(
    publication: &crate::mounting::UiMountedFramePublicationReceipt,
    mounted: &crate::mounting::WorthUiMountedSessionState,
    instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    basis: &crate::mounting::UiMountedIdentityBasis,
) -> Option<crate::runtime::scroll::UiScrollAnchor> {
    let presentation = publication
        .presentation_for_surface(basis.semantic_surface_identity())
        .map(|displayed| displayed.basis())?;
    let hit_test = mounted.interaction_hit_test_basis(presentation).ok()?;
    let row = hit_test
        .rows()
        .iter()
        .find(|row| row.mounted_instance() == instance)?;
    let [x, y, _, _] = row.bounds().adopted().components();
    crate::runtime::scroll::UiScrollAnchor::new(
        crate::runtime::scroll::UiScrollAnchorIdentity::mounted(instance),
        presentation.binding(),
        signed_subpixels(x)?.max(0),
        signed_subpixels(y)?.max(0),
    )
}

fn signed_subpixels(value: f32) -> Option<i64> {
    let scaled = f64::from(value)
        * worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
    (scaled.is_finite() && scaled >= i64::MIN as f64 && scaled <= i64::MAX as f64)
        .then(|| scaled.round() as i64)
}
