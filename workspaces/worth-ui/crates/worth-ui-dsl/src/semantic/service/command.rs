#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiCommandScope {
    Application,
    Surface,
    ActiveRegion,
    FocusedControl,
    ActivePortal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiCommandModifier {
    Primary,
    Shift,
    Control,
    Alt,
    Meta,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiCommandKey {
    Letter(char),
    Digit(u8),
    Function(u8),
    Enter,
    Escape,
    Tab,
    Space,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiCommandShortcutStrokeSpec {
    modifiers: Box<[WorthUiCommandModifier]>,
    key: WorthUiCommandKey,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiCommandDeclaration {
    identity: Box<str>,
    shortcut: Box<[WorthUiCommandShortcutStrokeSpec]>,
    scope: WorthUiCommandScope,
    scope_identity: Option<Box<str>>,
}

impl WorthUiCommandDeclaration {
    pub(super) fn parse(
        identity: &str,
        words: &[super::Word],
    ) -> Result<Self, super::WorthUiServiceDeclarationParseError> {
        super::validate_clauses(
            words,
            &[
                super::ClauseRule::List("shortcut"),
                super::ClauseRule::Single("scope"),
                super::ClauseRule::Single("binding"),
            ],
        )?;
        let shortcut_words = super::words_until(words, "shortcut", &["scope", "binding"])?;
        let shortcut = parse_shortcut(&shortcut_words)?;
        let scope =
            match super::one_value(words, "scope")? {
                "application" => WorthUiCommandScope::Application,
                "surface" => WorthUiCommandScope::Surface,
                "active_region" => return Err(super::invalid(
                    "command scope",
                    "active_region",
                    "use application, surface, focused_control, or active_portal until active-region runtime authority is available",
                )),
                "focused_control" => WorthUiCommandScope::FocusedControl,
                "active_portal" => WorthUiCommandScope::ActivePortal,
                value => return Err(super::invalid(
                    "command scope",
                    value,
                    "use application, surface, focused_control, or active_portal",
                )),
            };
        let scope_identity = super::optional_value(words, "binding").map(Box::from);
        if matches!(
            scope,
            WorthUiCommandScope::FocusedControl | WorthUiCommandScope::ActivePortal
        ) && scope_identity.is_none()
        {
            return Err(super::missing(
                "command binding",
                "bind the route to one authored control or portal semantic identity",
            ));
        }
        if matches!(
            scope,
            WorthUiCommandScope::Application | WorthUiCommandScope::Surface
        ) && scope_identity.is_some()
        {
            return Err(super::invalid(
                "command binding",
                scope_identity.as_deref().unwrap_or_default(),
                "application and surface routes do not consume a control or portal binding",
            ));
        }
        Ok(Self {
            identity: identity.into(),
            shortcut,
            scope,
            scope_identity,
        })
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn shortcut(&self) -> &[WorthUiCommandShortcutStrokeSpec] {
        &self.shortcut
    }
    pub const fn scope(&self) -> WorthUiCommandScope {
        self.scope
    }
    pub fn scope_identity(&self) -> Option<&str> {
        self.scope_identity.as_deref()
    }
    pub(super) fn canonical_text(&self) -> String {
        let shortcut = self
            .shortcut
            .iter()
            .map(WorthUiCommandShortcutStrokeSpec::canonical_text)
            .collect::<Vec<_>>()
            .join(" then ");
        format!(
            "command:{}:{}:{:?}:{}",
            self.identity,
            shortcut,
            self.scope,
            self.scope_identity.as_deref().unwrap_or("-")
        )
    }
}

impl WorthUiCommandShortcutStrokeSpec {
    pub fn modifiers(&self) -> &[WorthUiCommandModifier] {
        &self.modifiers
    }
    pub const fn key(&self) -> WorthUiCommandKey {
        self.key
    }
    fn canonical_text(&self) -> String {
        self.modifiers
            .iter()
            .map(|modifier| modifier.canonical_name().to_owned())
            .chain([self.key.canonical_name()])
            .collect::<Vec<_>>()
            .join("+")
    }
}

fn parse_shortcut(
    words: &[super::Word],
) -> Result<Box<[WorthUiCommandShortcutStrokeSpec]>, super::WorthUiServiceDeclarationParseError> {
    let mut strokes = Vec::new();
    let mut current = Vec::new();
    for word in words {
        match word {
            super::Word::Text(text) if text == "then" => {
                strokes.push(parse_stroke(&current)?);
                current.clear();
            }
            other => current.push(other.clone()),
        }
    }
    strokes.push(parse_stroke(&current)?);
    if strokes.len() > 2 {
        return Err(super::invalid(
            "command shortcut",
            "more than two strokes",
            "use one or two strokes",
        ));
    }
    Ok(strokes.into_boxed_slice())
}

fn parse_stroke(
    words: &[super::Word],
) -> Result<WorthUiCommandShortcutStrokeSpec, super::WorthUiServiceDeclarationParseError> {
    let parts = words
        .iter()
        .filter_map(|word| match word {
            super::Word::Text(text) => Some(text.as_str()),
            super::Word::Plus => None,
        })
        .collect::<Vec<_>>();
    let Some((key, modifiers)) = parts.split_last() else {
        return Err(super::missing("command shortcut", "declare a key"));
    };
    let parsed_modifiers = modifiers
        .iter()
        .map(|modifier| WorthUiCommandModifier::parse(modifier))
        .collect::<Result<Vec<_>, _>>()?;
    if parsed_modifiers
        .iter()
        .enumerate()
        .any(|(index, modifier)| parsed_modifiers[index + 1..].contains(modifier))
    {
        return Err(super::invalid(
            "command modifiers",
            "duplicate modifier",
            "declare each modifier once",
        ));
    }
    if parsed_modifiers.contains(&WorthUiCommandModifier::Primary)
        && (parsed_modifiers.contains(&WorthUiCommandModifier::Control)
            || parsed_modifiers.contains(&WorthUiCommandModifier::Meta))
    {
        return Err(super::invalid(
            "command modifiers",
            "Primary with Control or Meta",
            "use Primary alone for the platform command modifier",
        ));
    }
    let modifiers = parsed_modifiers.into_boxed_slice();
    let key = WorthUiCommandKey::parse(key)?;
    Ok(WorthUiCommandShortcutStrokeSpec { modifiers, key })
}

impl WorthUiCommandModifier {
    fn parse(value: &str) -> Result<Self, super::WorthUiServiceDeclarationParseError> {
        match value {
            "Primary" => Ok(Self::Primary),
            "Shift" => Ok(Self::Shift),
            "Control" => Ok(Self::Control),
            "Alt" => Ok(Self::Alt),
            "Meta" => Ok(Self::Meta),
            other => Err(super::invalid(
                "command modifier",
                other,
                "use Primary, Shift, Control, Alt, or Meta",
            )),
        }
    }

    const fn canonical_name(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Shift => "Shift",
            Self::Control => "Control",
            Self::Alt => "Alt",
            Self::Meta => "Meta",
        }
    }
}

impl WorthUiCommandKey {
    fn parse(value: &str) -> Result<Self, super::WorthUiServiceDeclarationParseError> {
        if value.len() == 1 {
            let byte = value.as_bytes()[0];
            if byte.is_ascii_alphabetic() {
                return Ok(Self::Letter(char::from(byte.to_ascii_uppercase())));
            }
            if byte.is_ascii_digit() {
                return Ok(Self::Digit(byte - b'0'));
            }
        }
        if let Some(number) = value
            .strip_prefix('F')
            .and_then(|number| number.parse::<u8>().ok())
            && (1..=35).contains(&number)
        {
            return Ok(Self::Function(number));
        }
        match value {
            "Enter" => Ok(Self::Enter),
            "Escape" => Ok(Self::Escape),
            "Tab" => Ok(Self::Tab),
            "Space" => Ok(Self::Space),
            "ArrowUp" => Ok(Self::ArrowUp),
            "ArrowDown" => Ok(Self::ArrowDown),
            "ArrowLeft" => Ok(Self::ArrowLeft),
            "ArrowRight" => Ok(Self::ArrowRight),
            other => Err(super::invalid(
                "command key",
                other,
                "use a canonical named key",
            )),
        }
    }

    pub fn canonical_name(self) -> String {
        match self {
            Self::Letter(letter) => letter.to_string(),
            Self::Digit(digit) => digit.to_string(),
            Self::Function(number) => format!("F{number}"),
            Self::Enter => "Enter".to_owned(),
            Self::Escape => "Escape".to_owned(),
            Self::Tab => "Tab".to_owned(),
            Self::Space => "Space".to_owned(),
            Self::ArrowUp => "ArrowUp".to_owned(),
            Self::ArrowDown => "ArrowDown".to_owned(),
            Self::ArrowLeft => "ArrowLeft".to_owned(),
            Self::ArrowRight => "ArrowRight".to_owned(),
        }
    }
}
