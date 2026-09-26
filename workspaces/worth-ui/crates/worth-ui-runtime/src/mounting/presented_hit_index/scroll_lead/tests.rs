use super::*;
use worth_ui_host_contract::*;

struct Fixture {
    index: UiPresentedHitIndex,
    binding: UiSurfaceBindingGeneration,
    surface: UiSemanticSurfaceIdentity,
    instance: UiMountedInstanceIdentity,
}

fn fixture() -> Fixture {
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let mut index = UiPresentedHitIndex::default();
    index.replace_base(
        instance,
        Some(super::super::tests::row(
            issuer,
            binding,
            surface,
            instance,
            0,
            [0.0, 100.0, 10.0, 10.0],
        )),
    );
    Fixture {
        index,
        binding,
        surface,
        instance,
    }
}

/// A move of `points` up the page from the origin.
fn up(points: i64) -> UiHitScrollMove {
    UiHitScrollMove::new(
        crate::mounting::presentation::UiScrollPoseShift::between(
            crate::runtime::scroll::UiScrollOffset::origin(),
            crate::runtime::scroll::UiScrollOffset::new(
                0,
                points * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
            )
            .unwrap(),
        ),
        crate::mounting::UiHitAncestorClip::Unclipped,
    )
}

impl Fixture {
    fn y(&self) -> f32 {
        self.index
            .for_instance(self.binding, self.instance)
            .0
            .unwrap()
            .bounds()
            .platform_box()
            .y()
    }

    fn lead(&mut self, lead: UiHitScrollMove) -> UiHitTestSpatialWork {
        let leads = [(self.instance, lead)];
        self.index
            .move_by_scroll(self.binding, &leads, UiHitScrollStanding::Leading)
    }

    fn commit(
        &mut self,
        moves: &[(UiMountedInstanceIdentity, UiHitScrollMove)],
    ) -> UiHitTestSpatialWork {
        self.index
            .move_by_scroll(self.binding, moves, UiHitScrollStanding::Committed)
    }
}

#[test]
fn a_later_lead_replaces_the_one_before_it() {
    let mut led = fixture();
    led.lead(up(20));
    assert_eq!(led.y(), 80.0);
    led.lead(up(30));
    assert_eq!(led.y(), 70.0, "leads replace, never accumulate");
    assert_eq!(
        led.index
            .displayed_scroll_translation(led.binding, led.instance)
            .0,
        Some(up(30))
    );
}

#[test]
fn the_commit_a_lead_showed_leaves_the_row_where_the_lead_put_it() {
    let mut led = fixture();
    led.lead(up(20));
    let paid = led.commit(&[(led.instance, up(20))]);
    assert_eq!(led.y(), 80.0);
    assert_eq!(paid.scroll_rows_displaced(), 0);
    // The lead is retired, not stacked on the commit it showed.
    led.lead(up(5));
    assert_eq!(led.y(), 75.0);
}

#[test]
fn a_commit_that_moves_nothing_retires_the_lead() {
    let mut led = fixture();
    led.lead(up(20));
    led.commit(&[]);
    assert_eq!(led.y(), 100.0);
    assert!(led.index.scroll_leads.is_empty());
}

#[test]
fn a_republished_row_drops_its_lead() {
    let mut led = fixture();
    led.lead(up(20));
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    led.index.replace_base(
        led.instance,
        Some(super::super::tests::row(
            issuer,
            led.binding,
            led.surface,
            led.instance,
            0,
            [0.0, 100.0, 10.0, 10.0],
        )),
    );
    assert_eq!(led.y(), 100.0);
    assert!(led.index.scroll_leads.is_empty());
}

#[test]
fn a_frame_that_reuses_a_led_row_keeps_its_lead() {
    let mut led = fixture();
    let mut reused = Fixture {
        index: led.index.clone(),
        ..led
    };
    led.lead(up(20));
    reused.index.inherit_scroll_leads(&led.index, led.binding);
    assert_eq!(reused.y(), 80.0);
    reused.commit(&[]);
    assert_eq!(
        reused.y(),
        100.0,
        "the reused frame retires the lead it kept"
    );
}

#[test]
fn a_frame_that_republished_a_led_row_does_not_take_its_lead() {
    let mut led = fixture();
    let mut reused = Fixture {
        index: led.index.clone(),
        ..led
    };
    led.lead(up(20));
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    reused.index.replace_base(
        reused.instance,
        Some(super::super::tests::row(
            issuer,
            reused.binding,
            reused.surface,
            reused.instance,
            0,
            [0.0, 100.0, 10.0, 10.0],
        )),
    );
    reused.index.inherit_scroll_leads(&led.index, led.binding);
    assert_eq!(reused.y(), 100.0, "a republished row measures from itself");
    assert!(reused.index.scroll_leads.is_empty());
}

#[test]
fn a_frame_whose_row_committed_another_pose_does_not_take_its_lead() {
    let mut led = fixture();
    let mut reused = Fixture {
        index: led.index.clone(),
        ..led
    };
    led.lead(up(20));
    reused.commit(&[(reused.instance, up(10))]);
    reused.index.inherit_scroll_leads(&led.index, led.binding);
    assert_eq!(
        reused.y(),
        90.0,
        "the lead measured from poses this row left"
    );
    assert!(reused.index.scroll_leads.is_empty());
}

#[test]
fn a_frame_takes_no_lead_for_another_binding() {
    let mut led = fixture();
    let mut reused = Fixture {
        index: led.index.clone(),
        ..led
    };
    led.lead(up(20));
    let other = UiSurfaceBindingGeneration::mint_unbound().unwrap();
    reused.index.inherit_scroll_leads(&led.index, other);
    assert_eq!(reused.y(), 100.0);
    assert!(reused.index.scroll_leads.is_empty());
}
