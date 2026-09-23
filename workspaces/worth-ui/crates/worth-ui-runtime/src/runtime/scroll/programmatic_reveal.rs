#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollRevealInterval {
    start_subpixels: i64,
    end_subpixels: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollRevealTarget {
    inline: UiScrollRevealInterval,
    block: UiScrollRevealInterval,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollViewportExtent {
    inline_subpixels: i64,
    block_subpixels: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollProgrammaticRevealRequest {
    chain: Vec<super::UiScrollChainEntry>,
    target: UiScrollRevealTarget,
    viewport: UiScrollViewportExtent,
    alignment: crate::declaration::UiScrollRevealAlignment,
}

impl UiScrollRevealInterval {
    pub(crate) const fn new(start_subpixels: i64, end_subpixels: i64) -> Option<Self> {
        if start_subpixels < 0 || end_subpixels < start_subpixels {
            None
        } else {
            Some(Self {
                start_subpixels,
                end_subpixels,
            })
        }
    }
}

impl UiScrollRevealTarget {
    pub(crate) const fn new(inline: UiScrollRevealInterval, block: UiScrollRevealInterval) -> Self {
        Self { inline, block }
    }
}

impl UiScrollViewportExtent {
    pub(crate) const fn new(inline_subpixels: i64, block_subpixels: i64) -> Option<Self> {
        if inline_subpixels <= 0 || block_subpixels <= 0 {
            None
        } else {
            Some(Self {
                inline_subpixels,
                block_subpixels,
            })
        }
    }
}

impl UiScrollProgrammaticRevealRequest {
    pub(crate) fn new(
        chain: Vec<super::UiScrollChainEntry>,
        target: UiScrollRevealTarget,
        viewport: UiScrollViewportExtent,
        alignment: crate::declaration::UiScrollRevealAlignment,
    ) -> Result<Self, super::UiScrollRouteDenial> {
        super::UiScrollDeltaRequest::new(
            chain.clone(),
            super::UiScrollDelta::new(0, 0),
            super::UiScrollDeltaCause::ProgrammaticReveal,
        )?;
        Ok(Self {
            chain,
            target,
            viewport,
            alignment,
        })
    }

    pub(super) fn chain(&self) -> &[super::UiScrollChainEntry] {
        &self.chain
    }
}

impl super::UiScrollRuntimeState {
    /// Place the chain so the requested interval is inside the viewport.
    ///
    /// A reveal is direct authority over the accepted offset, the same as a
    /// thumb placed on a track: it says where the content is, not where it is
    /// heading. So a settle already travelling toward a wheel target is an
    /// intention this request has just overruled, and every owner the route
    /// moved retires its target rather than resume dragging the content off
    /// what the reveal put on screen.
    ///
    /// The retirement happens after the route, because the route is the
    /// fallible half: a refused reveal moves nothing and must therefore leave
    /// the settle it would have replaced exactly as it found it.
    pub(crate) fn reveal(
        &mut self,
        request: UiScrollProgrammaticRevealRequest,
    ) -> Result<super::UiScrollRouteReceipt, super::UiScrollRouteDenial> {
        let first = request.chain()[0];
        let (current, bounds, axes) = self.owner_geometry(first.owner(), first.incarnation())?;
        let desired = desired_offset(
            current,
            bounds,
            axes,
            request.target,
            request.viewport,
            request.alignment,
        );
        let delta = super::UiScrollDelta::new(
            desired.inline_subpixels() - current.inline_subpixels(),
            desired.block_subpixels() - current.block_subpixels(),
        );
        let receipt = self.route(super::UiScrollDeltaRequest::new(
            request.chain,
            delta,
            super::UiScrollDeltaCause::ProgrammaticReveal,
        )?)?;
        for transition in receipt.transitions() {
            self.retire_transition(transition.owner());
        }
        Ok(receipt)
    }
}

fn desired_offset(
    current: super::UiScrollOffset,
    bounds: super::UiScrollBounds,
    axes: super::UiScrollAxes,
    target: UiScrollRevealTarget,
    viewport: UiScrollViewportExtent,
    alignment: crate::declaration::UiScrollRevealAlignment,
) -> super::UiScrollOffset {
    let inline = if axes.accepts_inline() {
        aligned_axis(
            current.inline_subpixels(),
            target.inline,
            viewport.inline_subpixels,
            alignment,
        )
    } else {
        current.inline_subpixels()
    };
    let block = if axes.accepts_block() {
        aligned_axis(
            current.block_subpixels(),
            target.block,
            viewport.block_subpixels,
            alignment,
        )
    } else {
        current.block_subpixels()
    };
    bounds.clamp_subpixels(inline, block)
}

fn aligned_axis(
    current: i64,
    target: UiScrollRevealInterval,
    viewport: i64,
    alignment: crate::declaration::UiScrollRevealAlignment,
) -> i64 {
    match alignment {
        crate::declaration::UiScrollRevealAlignment::End => {
            target.end_subpixels.saturating_sub(viewport)
        }
        crate::declaration::UiScrollRevealAlignment::Nearest => {
            let viewport_end = current.saturating_add(viewport);
            if target.start_subpixels < current {
                target.start_subpixels
            } else if target.end_subpixels > viewport_end {
                target.end_subpixels.saturating_sub(viewport)
            } else {
                current
            }
        }
    }
}
