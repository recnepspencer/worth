/// Non-current profile identity reserved for native appearance qualification.
///
/// This type intentionally has no preparation, selection, or publication
/// operation. The runtime continues to own the live v1 profile binding until
/// the later atomic protocol cutover.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiNativeStagedProfileIdentity(&'static str);

impl UiNativeStagedProfileIdentity {
    pub const WORTH_UI_WINDOWS_DX12_V2: Self = Self("worth-ui-windows-dx12-v2");

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

pub const WORTH_UI_NATIVE_NEXT_PROFILE_IDENTITY: UiNativeStagedProfileIdentity =
    UiNativeStagedProfileIdentity::WORTH_UI_WINDOWS_DX12_V2;
