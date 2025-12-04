use defmt::Format;
use usbd_hid::descriptor::KeyboardUsage;

/// Short‑hand enum that mirrors every variant of `KeyboardUsage`.
/// The discriminants are exactly the same HID usage codes, so you can use
/// `KC` wherever the original values are required while keeping the terse names.
#[derive(PartialEq, Eq, Debug, Clone, Copy, Format)]
pub enum KC {
    // ------------------------------------------------------------------------
    // 0x00: Reserved
    /// Keyboard ErrorRollOver (Footnote 1)
    ERO = KeyboardUsage::KeyboardErrorRollOver as isize,
    /// Keyboard POSTFail (Footnote 1)
    PF = KeyboardUsage::KeyboardPOSTFail as isize,
    /// Keyboard ErrorUndefined (Footnote 1)
    EU = KeyboardUsage::KeyboardErrorUndefined as isize,

    // ------------------------------------------------------------------------
    // 0x04‑0x1D: Alphanumeric keys
    /// Keyboard a and A (Footnote 2)
    Aa = KeyboardUsage::KeyboardAa as isize,
    /// Keyboard b and B
    Bb = KeyboardUsage::KeyboardBb as isize,
    /// Keyboard c and C (Footnote 2)
    Cc = KeyboardUsage::KeyboardCc as isize,
    /// Keyboard d and D
    Dd = KeyboardUsage::KeyboardDd as isize,
    /// Keyboard e and E
    Ee = KeyboardUsage::KeyboardEe as isize,
    /// Keyboard f and F
    Ff = KeyboardUsage::KeyboardFf as isize,
    /// Keyboard g and G
    Gg = KeyboardUsage::KeyboardGg as isize,
    /// Keyboard h and H
    Hh = KeyboardUsage::KeyboardHh as isize,
    /// Keyboard i and I
    Ii = KeyboardUsage::KeyboardIi as isize,
    /// Keyboard j and J
    Jj = KeyboardUsage::KeyboardJj as isize,
    /// Keyboard k and K
    Kk = KeyboardUsage::KeyboardKk as isize,
    /// Keyboard l and L
    Ll = KeyboardUsage::KeyboardLl as isize,
    /// Keyboard m and M (Footnote 2)
    Mm = KeyboardUsage::KeyboardMm as isize,
    /// Keyboard n and N
    Nn = KeyboardUsage::KeyboardNn as isize,
    /// Keyboard o and O (Footnote 2)
    Oo = KeyboardUsage::KeyboardOo as isize,
    /// Keyboard p and P (Footnote 2)
    Pp = KeyboardUsage::KeyboardPp as isize,
    /// Keyboard q and Q (Footnote 2)
    Qq = KeyboardUsage::KeyboardQq as isize,
    /// Keyboard r and R
    Rr = KeyboardUsage::KeyboardRr as isize,
    /// Keyboard s and S
    Ss = KeyboardUsage::KeyboardSs as isize,
    /// Keyboard t and T
    Tt = KeyboardUsage::KeyboardTt as isize,
    /// Keyboard u and U
    Uu = KeyboardUsage::KeyboardUu as isize,
    /// Keyboard v and V
    Vv = KeyboardUsage::KeyboardVv as isize,
    /// Keyboard w and W (Footnote 2)
    Ww = KeyboardUsage::KeyboardWw as isize,
    /// Keyboard x and X (Footnote 2)
    Xx = KeyboardUsage::KeyboardXx as isize,
    /// Keyboard y and Y (Footnote 2)
    Yy = KeyboardUsage::KeyboardYy as isize,
    /// Keyboard z and Z (Footnote 2)
    Zz = KeyboardUsage::KeyboardZz as isize,

    // ------------------------------------------------------------------------
    // 0x1E‑0x27: Number row (with shifted symbols)
    /// Keyboard 1 and ! (Footnote 2)
    K1 = KeyboardUsage::Keyboard1Exclamation as isize,
    /// Keyboard 2 and @ (Footnote 2)
    K2 = KeyboardUsage::Keyboard2At as isize,
    /// Keyboard 3 and # (Footnote 2)
    K3 = KeyboardUsage::Keyboard3Hash as isize,
    /// Keyboard 4 and $ (Footnote 2)
    K4 = KeyboardUsage::Keyboard4Dollar as isize,
    /// Keyboard 5 and % (Footnote 2)
    K5 = KeyboardUsage::Keyboard5Percent as isize,
    /// Keyboard 6 and ^ (Footnote 2)
    K6 = KeyboardUsage::Keyboard6Caret as isize,
    /// Keyboard 7 and & (Footnote 2)
    K7 = KeyboardUsage::Keyboard7Ampersand as isize,
    /// Keyboard 8 and * (Footnote 2)
    K8 = KeyboardUsage::Keyboard8Asterisk as isize,
    /// Keyboard 9 and ( (Footnote 2)
    K9 = KeyboardUsage::Keyboard9OpenParens as isize,
    /// Keyboard 0 and ) (Footnote 2)
    K0 = KeyboardUsage::Keyboard0CloseParens as isize,

    // ------------------------------------------------------------------------
    // 0x28‑0x2C: Basic control keys
    /// Keyboard Return (ENTER) (Footnote 3)
    Enter = KeyboardUsage::KeyboardEnter as isize,
    /// Keyboard ESCAPE
    Escape = KeyboardUsage::KeyboardEscape as isize,
    /// Keyboard DELETE (Backspace) (Footnote 4)
    Backspace = KeyboardUsage::KeyboardBackspace as isize,
    /// Keyboard Tab
    Tab = KeyboardUsage::KeyboardTab as isize,
    /// Keyboard Spacebar
    Space = KeyboardUsage::KeyboardSpacebar as isize,

    // ------------------------------------------------------------------------
    // 0x2D‑0x35: Symbol keys
    /// Keyboard - and _ (Footnote 2)
    Dash = KeyboardUsage::KeyboardDashUnderscore as isize,
    /// Keyboard = and + (Footnote 2)
    Equal = KeyboardUsage::KeyboardEqualPlus as isize,
    /// Keyboard [ and { (Footnote 2)
    OpenBracket = KeyboardUsage::KeyboardOpenBracketBrace as isize,
    /// Keyboard ] and } (Footnote 2)
    CloseBracket = KeyboardUsage::KeyboardCloseBracketBrace as isize,
    /// Keyboard \ and |
    Bslash = KeyboardUsage::KeyboardBackslashBar as isize,
    /// Keyboard Non‑US # (Footnote 5)
    NonUSHash = KeyboardUsage::KeyboardNonUSHash as isize,
    /// Keyboard ; and : (Footnote 2)
    SemiColon = KeyboardUsage::KeyboardSemiColon as isize,
    /// Keyboard ' and " (Footnote 2)
    Quote = KeyboardUsage::KeyboardSingleDoubleQuote as isize,
    /// Keyboard ` and ~ (Footnote 2)
    BacktickTilde = KeyboardUsage::KeyboardBacktickTilde as isize,
    /// Keyboard , and < (Footnote 2)
    Comma = KeyboardUsage::KeyboardCommaLess as isize,
    /// Keyboard . and > (Footnote 2)
    Period = KeyboardUsage::KeyboardPeriodGreater as isize,
    /// Keyboard / and ? (Footnote 2)
    Fslash = KeyboardUsage::KeyboardSlashQuestion as isize,
    /// Keyboard Caps Lock (Footnote 6)
    CapsLock = KeyboardUsage::KeyboardCapsLock as isize,

    // ------------------------------------------------------------------------
    // 0x3A‑0x45: Function keys
    F1 = KeyboardUsage::KeyboardF1 as isize,
    F2 = KeyboardUsage::KeyboardF2 as isize,
    F3 = KeyboardUsage::KeyboardF3 as isize,
    F4 = KeyboardUsage::KeyboardF4 as isize,
    F5 = KeyboardUsage::KeyboardF5 as isize,
    F6 = KeyboardUsage::KeyboardF6 as isize,
    F7 = KeyboardUsage::KeyboardF7 as isize,
    F8 = KeyboardUsage::KeyboardF8 as isize,
    F9 = KeyboardUsage::KeyboardF9 as isize,
    F10 = KeyboardUsage::KeyboardF10 as isize,
    F11 = KeyboardUsage::KeyboardF11 as isize,
    F12 = KeyboardUsage::KeyboardF12 as isize,
    F13 = KeyboardUsage::KeyboardF13 as isize,
    F14 = KeyboardUsage::KeyboardF14 as isize,
    F15 = KeyboardUsage::KeyboardF15 as isize,
    F16 = KeyboardUsage::KeyboardF16 as isize,
    F17 = KeyboardUsage::KeyboardF17 as isize,
    F18 = KeyboardUsage::KeyboardF18 as isize,
    F19 = KeyboardUsage::KeyboardF19 as isize,
    F20 = KeyboardUsage::KeyboardF20 as isize,
    F21 = KeyboardUsage::KeyboardF21 as isize,
    F22 = KeyboardUsage::KeyboardF22 as isize,
    F23 = KeyboardUsage::KeyboardF23 as isize,
    F24 = KeyboardUsage::KeyboardF24 as isize,

    // ------------------------------------------------------------------------
    // 0x46‑0x52: System / navigation keys
    /// Keyboard PrintScreen (Footnote 7)
    PrintS = KeyboardUsage::KeyboardPrintScreen as isize,
    /// Keyboard ScrollLock (Footnote 6)
    ScrollLock = KeyboardUsage::KeyboardScrollLock as isize,
    /// Keyboard Pause (Footnote 7)
    Pause = KeyboardUsage::KeyboardPause as isize,
    /// Keyboard Insert (Footnote 7)
    Insert = KeyboardUsage::KeyboardInsert as isize,
    /// Keyboard Home (Footnote 7)
    Home = KeyboardUsage::KeyboardHome as isize,
    /// Keyboard PageUp (Footnote 7)
    PageUp = KeyboardUsage::KeyboardPageUp as isize,
    /// Keyboard Delete Forward (Footnote 7, 8)
    Delete = KeyboardUsage::KeyboardDelete as isize,
    /// Keyboard End (Footnote 7)
    End = KeyboardUsage::KeyboardEnd as isize,
    /// Keyboard PageDown (Footnote 7)
    PageDown = KeyboardUsage::KeyboardPageDown as isize,
    /// Keyboard RightArrow (Footnote 7)
    RightArr = KeyboardUsage::KeyboardRightArrow as isize,
    /// Keyboard LeftArrow (Footnote 7)
    LeftArr = KeyboardUsage::KeyboardLeftArrow as isize,
    /// Keyboard DownArrow (Footnote 7)
    DownArr = KeyboardUsage::KeyboardDownArrow as isize,
    /// Keyboard UpArrow (Footnote 7)
    UpArr = KeyboardUsage::KeyboardUpArrow as isize,

    // ------------------------------------------------------------------------
    // 0x53‑0x58: Keypad basics
    /// Keypad Num Lock and Clear (Footnote 6)
    NumLock = KeyboardUsage::KeypadNumLock as isize,
    /// Keypad / (Footnote 7)
    KeypadDivide = KeyboardUsage::KeypadDivide as isize,
    /// Keypad *
    KeypadMultiply = KeyboardUsage::KeypadMultiply as isize,
    /// Keypad -
    KMinus = KeyboardUsage::KeypadMinus as isize,
    /// Keypad +
    KeypadPlus = KeyboardUsage::KeypadPlus as isize,
    /// Keypad ENTER (Footnote 3)
    KeypadEnter = KeyboardUsage::KeypadEnter as isize,

    // ------------------------------------------------------------------------
    // 0x59‑0x63: Keypad extended keys
    /// Keypad 1 and End
    Keypad1End = KeyboardUsage::Keypad1End as isize,
    /// Keypad 2 and DownArrow
    Keypad2DownArrow = KeyboardUsage::Keypad2DownArrow as isize,
    /// Keypad 3 and PageDown
    Keypad3PageDown = KeyboardUsage::Keypad3PageDown as isize,
    /// Keypad 4 and LeftArrow
    Keypad4LeftArrow = KeyboardUsage::Keypad4LeftArrow as isize,
    /// Keypad 5
    Keypad5 = KeyboardUsage::Keypad5 as isize,
    /// Keypad 6 and RightArrow
    Keypad6RightArrow = KeyboardUsage::Keypad6RightArrow as isize,
    /// Keypad 7 and Home
    Keypad7Home = KeyboardUsage::Keypad7Home as isize,
    /// Keypad 8 and UpArrow
    Keypad8UpArrow = KeyboardUsage::Keypad8UpArrow as isize,
    /// Keypad 9 and PageUp
    Keypad9PageUp = KeyboardUsage::Keypad9PageUp as isize,
    /// Keypad 0 and Insert
    Keypad0Insert = KeyboardUsage::Keypad0Insert as isize,
    /// Keypad . and Delete
    KeypadPeriodDelete = KeyboardUsage::KeypadPeriodDelete as isize,

    // ------------------------------------------------------------------------
    // 0x64‑0x65: Miscellaneous keys
    /// Keyboard Non‑US \ and | (Footnote 9, 10)
    USSlash = KeyboardUsage::KeyboardNonUSSlash as isize,
    /// Keyboard Application (Footnote 11)
    Application = KeyboardUsage::KeyboardApplication as isize,
    /// Keyboard Power (Footnote 1)
    Power = KeyboardUsage::KeyboardPower as isize,

    // ------------------------------------------------------------------------
    // 0x66‑0x67: Keypad extra
    /// Keypad =
    KeypadEqual = KeyboardUsage::KeypadEqual as isize,

    // ------------------------------------------------------------------------
    // 0x74‑0x7D: System control keys
    Execute = KeyboardUsage::KeyboardExecute as isize,
    Help = KeyboardUsage::KeyboardHelp as isize,
    Menu = KeyboardUsage::KeyboardMenu as isize,
    Select = KeyboardUsage::KeyboardSelect as isize,
    Stop = KeyboardUsage::KeyboardStop as isize,
    Again = KeyboardUsage::KeyboardAgain as isize,
    Undo = KeyboardUsage::KeyboardUndo as isize,
    Cut = KeyboardUsage::KeyboardCut as isize,
    Copy = KeyboardUsage::KeyboardCopy as isize,
    Paste = KeyboardUsage::KeyboardPaste as isize,
    Find = KeyboardUsage::KeyboardFind as isize,
    Mute = KeyboardUsage::KeyboardMute as isize,
    VolumeUp = KeyboardUsage::KeyboardVolumeUp as isize,
    VolumeDown = KeyboardUsage::KeyboardVolumeDown as isize,

    // ------------------------------------------------------------------------
    // 0x7E‑0x84: Locking keys
    LockingCapsLock = KeyboardUsage::KeyboardLockingCapsLock as isize,
    LockingNumLock = KeyboardUsage::KeyboardLockingNumLock as isize,
    LockingScrollLock = KeyboardUsage::KeyboardLockingScrollLock as isize,

    // ------------------------------------------------------------------------
    // 0x85‑0x86: Keypad punctuation
    KeypadComma = KeyboardUsage::KeypadComma as isize,
    KeypadEqualSign = KeyboardUsage::KeypadEqualSign as isize,

    // ------------------------------------------------------------------------
    // 0x87‑0x8F: International keys
    International1 = KeyboardUsage::KeyboardInternational1 as isize,
    International2 = KeyboardUsage::KeyboardInternational2 as isize,
    International3 = KeyboardUsage::KeyboardInternational3 as isize,
    International4 = KeyboardUsage::KeyboardInternational4 as isize,
    International5 = KeyboardUsage::KeyboardInternational5 as isize,
    International6 = KeyboardUsage::KeyboardInternational6 as isize,
    International7 = KeyboardUsage::KeyboardInternational7 as isize,
    International8 = KeyboardUsage::KeyboardInternational8 as isize,
    International9 = KeyboardUsage::KeyboardInternational9 as isize,

    // ------------------------------------------------------------------------
    // 0x90‑0x98: Language keys
    LANG1 = KeyboardUsage::KeyboardLANG1 as isize,
    LANG2 = KeyboardUsage::KeyboardLANG2 as isize,
    LANG3 = KeyboardUsage::KeyboardLANG3 as isize,
    LANG4 = KeyboardUsage::KeyboardLANG4 as isize,
    LANG5 = KeyboardUsage::KeyboardLANG5 as isize,
    LANG6 = KeyboardUsage::KeyboardLANG6 as isize,
    LANG7 = KeyboardUsage::KeyboardLANG7 as isize,
    LANG8 = KeyboardUsage::KeyboardLANG8 as isize,
    LANG9 = KeyboardUsage::KeyboardLANG9 as isize,

    // ------------------------------------------------------------------------
    // 0x99‑0x9C: Misc system keys
    AlternateErase = KeyboardUsage::KeyboardAlternateErase as isize,
    SysReqAttention = KeyboardUsage::KeyboardSysReqAttention as isize,
    Cancel = KeyboardUsage::KeyboardCancel as isize,
    Clear = KeyboardUsage::KeyboardClear as isize,

    // ------------------------------------------------------------------------
    // 0x9D‑0xA4: Navigation / selection keys
    Prior = KeyboardUsage::KeyboardPrior as isize,
    Return = KeyboardUsage::KeyboardReturn as isize,
    Separator = KeyboardUsage::KeyboardSeparator as isize,
    Out = KeyboardUsage::KeyboardOut as isize,
    Oper = KeyboardUsage::KeyboardOper as isize,
    ClearAgain = KeyboardUsage::KeyboardClearAgain as isize,
    CrSelProps = KeyboardUsage::KeyboardCrSelProps as isize,
    ExSel = KeyboardUsage::KeyboardExSel as isize,

    // ------------------------------------------------------------------------
    // 0xB0‑0xBF: Keypad numeric extensions
    Keypad00 = KeyboardUsage::Keypad00 as isize,
    Keypad000 = KeyboardUsage::Keypad000 as isize,
    ThousandsSeparator = KeyboardUsage::ThousandsSeparator as isize,
    DecimalSeparator = KeyboardUsage::DecimalSeparator as isize,
    CurrencyUnit = KeyboardUsage::CurrencyUnit as isize,
    CurrencySubunit = KeyboardUsage::CurrencySubunit as isize,
    OpenParens = KeyboardUsage::KeypadOpenParens as isize,
    CloseParens = KeyboardUsage::KeypadCloseParens as isize,
    OpenBrace = KeyboardUsage::KeypadOpenBrace as isize,
    CloseBrace = KeyboardUsage::KeypadCloseBrace as isize,
    KeypadTab = KeyboardUsage::KeypadTab as isize,
    KeypadBackspace = KeyboardUsage::KeypadBackspace as isize,
    A = KeyboardUsage::KeypadA as isize,
    B = KeyboardUsage::KeypadB as isize,
    C = KeyboardUsage::KeypadC as isize,
    D = KeyboardUsage::KeypadD as isize,

    // ------------------------------------------------------------------------
    // 0xC0‑0xCA: Keypad logical / bitwise ops
    E = KeyboardUsage::KeypadE as isize,
    F = KeyboardUsage::KeypadF as isize,
    BitwiseXor = KeyboardUsage::KeypadBitwiseXor as isize,
    LogicalXor = KeyboardUsage::KeypadLogicalXor as isize,
    Modulo = KeyboardUsage::KeypadModulo as isize,
    LShift = KeyboardUsage::KeypadLeftShift as isize,
    RightShift = KeyboardUsage::KeypadRightShift as isize,
    BitwiseAnd = KeyboardUsage::KeypadBitwiseAnd as isize,
    LogicalAnd = KeyboardUsage::KeypadLogicalAnd as isize,
    BitwiseOr = KeyboardUsage::KeypadBitwiseOr as isize,
    LogicalOr = KeyboardUsage::KeypadLogicalOr as isize,
    Colon = KeyboardUsage::KeypadColon as isize,
    Hash = KeyboardUsage::KeypadHash as isize,
    KeypadSpace = KeyboardUsage::KeypadSpace as isize,
    At = KeyboardUsage::KeypadAt as isize,
    Exclamation = KeyboardUsage::KeypadExclamation as isize,

    // ------------------------------------------------------------------------
    // 0xD0‑0xD9: Keypad memory functions
    MemoryStore = KeyboardUsage::KeypadMemoryStore as isize,
    MemoryRecall = KeyboardUsage::KeypadMemoryRecall as isize,
    MemoryClear = KeyboardUsage::KeypadMemoryClear as isize,
    MemoryAdd = KeyboardUsage::KeypadMemoryAdd as isize,

    MemorySubtract = KeyboardUsage::KeypadMemorySubtract as isize,
    MemoryMultiply = KeyboardUsage::KeypadMemoryMultiply as isize,
    MemoryDivide = KeyboardUsage::KeypadMemoryDivide as isize,
    PositiveNegative = KeyboardUsage::KeypadPositiveNegative as isize,
    KeypadClear = KeyboardUsage::KeypadClear as isize,
    ClearEntry = KeyboardUsage::KeypadClearEntry as isize,
    Binary = KeyboardUsage::KeypadBinary as isize,
    Octal = KeyboardUsage::KeypadOctal as isize,
    Decimal = KeyboardUsage::KeypadDecimal as isize,
    Hexadecimal = KeyboardUsage::KeypadHexadecimal as isize,

    // ------------------------------------------------------------------------
    // 0xE0‑0xE7: Modifier keys
    LCtrl = KeyboardUsage::KeyboardLeftControl as isize,
    LeftShift = KeyboardUsage::KeyboardLeftShift as isize,
    LAlt = KeyboardUsage::KeyboardLeftAlt as isize,
    LGUI = KeyboardUsage::KeyboardLeftGUI as isize,
    RCtrs = KeyboardUsage::KeyboardRightControl as isize,
    RShift = KeyboardUsage::KeyboardRightShift as isize,
    RAlt = KeyboardUsage::KeyboardRightAlt as isize,
    RGUI = KeyboardUsage::KeyboardRightGUI as isize,

    // ------------------------------------------------------------------------
    // 0xE8‑0xFF: Reserved / invalid values
    Reserved = KeyboardUsage::Reserved as isize,

    // -----------------------------------------------------------------------
    // Custom Internal Keycodes
    /// Layer 1
    L1 = 0xF0,
    /// Layer 2
    L2 = 0xF1,
    /// Layer 3
    L3 = 0xF2,
    /// Layer 4
    L4 = 0xF3,
    /// Layer 5
    L5 = 0xF4,
}

impl KC {
    pub fn get_modifier(&self) -> u8 {
        match self {
            KC::LCtrl => 0x01,
            KC::LShift => 0x02,
            KC::LAlt => 0x04,
            KC::LGUI => 0x08,
            // KC::RShift => {}
            // KC::RCtrs => {}
            // KC::RAlt => {}
            // KC::RGUI => {}
            _ => 0x00,
        }
    }

    pub fn get_layer(&self) -> u8 {
        match self {
            KC::L1 => 1,
            KC::L2 => 2,
            KC::L3 => 3,
            KC::L4 => 4,
            KC::L5 => 5,
            _ => 0,
        }
    }
}

pub enum KeyType {
    Combo,
    Macro,
    Modifier,
    Mouse,
    Key,
    Layer,
}

impl KeyType {
    pub fn check_type(key: &KC) -> KeyType {
        match *key {
            // // return Macro key type
            // KC::MaLP
            // | KC::MaRP
            // | KC::MaCp
            // | KC::MaPa
            // | KC::MaEx
            // | KC::MaAt
            // | KC::MaHs
            // | KC::MaDl
            // | KC::MaMd
            // | KC::MaCa
            // | KC::MaAmp
            // | KC::MaAst
            // | KC::MaSL
            // | KC::MaLB
            // | KC::MaRB
            // | KC::MaPipe => KeyType::Macro,

            // return Layer key type
            KC::L1 | KC::L2 | KC::L3 | KC::L4 | KC::L5 => KeyType::Layer,

            // return Modifier key type
            KC::LShift
            | KC::LCtrl
            | KC::LAlt
            | KC::LGUI
            | KC::RShift
            | KC::RCtrs
            | KC::RAlt
            | KC::RGUI => KeyType::Modifier,

            // // return Mouse key type
            // KC::MoGL
            // | KC::MoGD
            // | KC::MoGU
            // | KC::MoGR
            // | KC::MoLC
            // | KC::MoRC
            // | KC::MoSL
            // | KC::MoSR
            // | KC::MoSU
            // | KC::MoSD
            // | KC::MoCF
            // | KC::MoCN
            // | KC::MoCS => KeyType::Mouse,

            // return Combo key type
            // KC::ComboCtrlD => KeyType::Combo,
            _ => KeyType::Key,
        }
    }
}
