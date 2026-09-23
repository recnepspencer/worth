#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiScrollAnchorPolicy {
    StableKey,
    Clamp,
}

/// Which axes an authored scroll declaration shows chrome for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiScrollChromeAxes {
    Inline,
    Block,
    Both,
}

/// An authored chrome declaration: the axes, and the appearance roles that
/// paint the track and the thumb.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiScrollChromeDeclaration {
    axes: WorthUiScrollChromeAxes,
    track_role: Box<str>,
    thumb_role: Box<str>,
}

impl WorthUiScrollChromeDeclaration {
    pub const fn axes(&self) -> WorthUiScrollChromeAxes {
        self.axes
    }
    pub fn track_role(&self) -> &str {
        &self.track_role
    }
    pub fn thumb_role(&self) -> &str {
        &self.thumb_role
    }
}

/// How an authored scroll declaration answers one coarse wheel notch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiScrollWheelPolicy {
    /// The offset moves on the observation that carried the notch.
    Immediate,
    /// The offset becomes a target that settles over this many milliseconds
    /// measured from the latest notch; the runtime owns the admissible range.
    Smooth { settle_ticks: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiScrollDeclaration {
    identity: Box<str>,
    nested: bool,
    anchor: WorthUiScrollAnchorPolicy,
    line_extent_logical_points: Option<u16>,
    chrome: Option<WorthUiScrollChromeDeclaration>,
    wheel: WorthUiScrollWheelPolicy,
}

impl WorthUiScrollDeclaration {
    pub(super) fn parse(
        identity: &str,
        words: &[super::Word],
    ) -> Result<Self, super::WorthUiServiceDeclarationParseError> {
        super::validate_clauses(
            words,
            &[
                super::ClauseRule::Flag("nested"),
                super::ClauseRule::Single("anchor"),
                super::ClauseRule::Single("line_extent"),
                super::ClauseRule::Single("chrome"),
                super::ClauseRule::Single("track"),
                super::ClauseRule::Single("thumb"),
                super::ClauseRule::List("wheel"),
            ],
        )?;
        let anchor = match super::one_value(words, "anchor")? {
            "stable_key" => WorthUiScrollAnchorPolicy::StableKey,
            "clamp" => WorthUiScrollAnchorPolicy::Clamp,
            value => {
                return Err(super::invalid(
                    "scroll anchor",
                    value,
                    "use stable_key or clamp",
                ))
            }
        };
        Ok(Self {
            identity: identity.into(),
            nested: super::optional_flag(words, "nested"),
            anchor,
            line_extent_logical_points: line_extent(words)?,
            chrome: chrome(words)?,
            wheel: wheel(words)?,
        })
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub const fn nested(&self) -> bool {
        self.nested
    }
    pub const fn anchor(&self) -> WorthUiScrollAnchorPolicy {
        self.anchor
    }
    /// What one line of this region's content is worth, in logical points.
    pub const fn line_extent_logical_points(&self) -> Option<u16> {
        self.line_extent_logical_points
    }
    pub const fn chrome(&self) -> Option<&WorthUiScrollChromeDeclaration> {
        self.chrome.as_ref()
    }
    /// How this region answers a coarse wheel notch; immediate when unstated.
    pub const fn wheel(&self) -> WorthUiScrollWheelPolicy {
        self.wheel
    }

    pub(super) fn canonical_text(&self) -> String {
        format!(
            "scroll:{}:{}:{:?}:{}:{}:{}",
            self.identity,
            self.nested,
            self.anchor,
            canonical_line_extent(self.line_extent_logical_points),
            canonical_chrome(self.chrome.as_ref()),
            canonical_wheel(self.wheel),
        )
    }
}

/// The canonical spelling of a line extent, absent or present.
fn canonical_line_extent(logical_points: Option<u16>) -> String {
    match logical_points {
        None => "-".to_owned(),
        Some(points) => points.to_string(),
    }
}

/// The canonical spelling of a chrome declaration, absent or present.
///
/// Authoring order never reaches this text: the axes, the track and the thumb
/// are always spelled in this order, so a Rust author and a file author who
/// declare the same chrome produce the same canonical contract.
fn canonical_chrome(chrome: Option<&WorthUiScrollChromeDeclaration>) -> String {
    match chrome {
        None => "-".to_owned(),
        Some(chrome) => format!(
            "{:?}/{}/{}",
            chrome.axes, chrome.track_role, chrome.thumb_role
        ),
    }
}

fn line_extent(
    words: &[super::Word],
) -> Result<Option<u16>, super::WorthUiServiceDeclarationParseError> {
    let Some(value) = super::optional_value(words, "line_extent") else {
        return Ok(None);
    };
    match value.parse::<u16>() {
        Ok(0) | Err(_) => Err(super::invalid(
            "scroll line extent",
            value,
            "use a whole number of logical points between 1 and 65535",
        )),
        Ok(points) => Ok(Some(points)),
    }
}

fn chrome(
    words: &[super::Word],
) -> Result<Option<WorthUiScrollChromeDeclaration>, super::WorthUiServiceDeclarationParseError> {
    let axes = super::optional_value(words, "chrome");
    let track_role = super::optional_value(words, "track");
    let thumb_role = super::optional_value(words, "thumb");
    let (Some(axes), Some(track_role), Some(thumb_role)) = (axes, track_role, thumb_role) else {
        if axes.is_none() && track_role.is_none() && thumb_role.is_none() {
            return Ok(None);
        }
        return Err(super::missing(
            "scroll chrome",
            "declare chrome, track and thumb together or declare none of them",
        ));
    };
    let axes = match axes {
        "inline" => WorthUiScrollChromeAxes::Inline,
        "block" => WorthUiScrollChromeAxes::Block,
        "both" => WorthUiScrollChromeAxes::Both,
        value => {
            return Err(super::invalid(
                "scroll chrome axes",
                value,
                "use inline, block or both",
            ))
        }
    };
    if track_role == thumb_role {
        return Err(super::invalid(
            "scroll chrome",
            track_role,
            "name one appearance role for the track and another for the thumb",
        ));
    }
    Ok(Some(WorthUiScrollChromeDeclaration {
        axes,
        track_role: track_role.into(),
        thumb_role: thumb_role.into(),
    }))
}

/// The canonical spelling of a wheel policy. An unstated wheel spells the
/// same as a declared immediate one: both lower to the same contract.
fn canonical_wheel(wheel: WorthUiScrollWheelPolicy) -> String {
    match wheel {
        WorthUiScrollWheelPolicy::Immediate => "immediate".to_owned(),
        WorthUiScrollWheelPolicy::Smooth { settle_ticks } => format!("smooth/{settle_ticks}"),
    }
}

/// Every other clause of a scroll declaration, where a wheel value list ends.
const WHEEL_STOPS: &[&str] = &[
    "nested",
    "anchor",
    "line_extent",
    "chrome",
    "track",
    "thumb",
];

fn wheel(
    words: &[super::Word],
) -> Result<WorthUiScrollWheelPolicy, super::WorthUiServiceDeclarationParseError> {
    if !super::optional_flag(words, "wheel") {
        return Ok(WorthUiScrollWheelPolicy::Immediate);
    }
    let values = super::values_until(words, "wheel", WHEEL_STOPS)?;
    match values.as_slice() {
        ["immediate"] => Ok(WorthUiScrollWheelPolicy::Immediate),
        ["smooth", ticks] => match ticks.parse::<u32>() {
            Ok(0) | Err(_) => Err(super::invalid(
                "scroll wheel settle",
                ticks,
                "use a whole number of milliseconds greater than zero",
            )),
            Ok(settle_ticks) => Ok(WorthUiScrollWheelPolicy::Smooth { settle_ticks }),
        },
        _ => Err(super::invalid(
            "scroll wheel",
            &values.join(" "),
            "use immediate, or smooth followed by one settle horizon in milliseconds",
        )),
    }
}
