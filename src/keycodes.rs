/// Short‑hand enum that mirrors every variant of `KeyboardUsage`.
/// The discriminants are exactly the same HID usage codes, so you can use
/// `KC` wherever the original values are required while keeping the terse names.
pub enum KC {
    // ------------------------------------------------------------------------
    // 0x00: Reserved
    /// Keyboard ErrorRollOver (Footnote 1)
    ERO = KeyboardUsage::KeyboardErrorRollOver,
    /// Keyboard POSTFail (Footnote 1)
    PF = KeyboardUsage::KeyboardPOSTFail,
    /// Keyboard ErrorUndefined (Footnote 1)
    EU = KeyboardUsage::KeyboardErrorUndefined,

    // ------------------------------------------------------------------------
    // 0x04‑0x1D: Alphanumeric keys
    /// Keyboard a and A (Footnote 2)
    Aa = KeyboardUsage::KeyboardAa,
    /// Keyboard b and B
    Bb = KeyboardUsage::KeyboardBb,
    /// Keyboard c and C (Footnote 2)
    Cc = KeyboardUsage::KeyboardCc,
    /// Keyboard d and D
    Dd = KeyboardUsage::KeyboardDd,
    /// Keyboard e and E
    Ee = KeyboardUsage::KeyboardEe,
    /// Keyboard f and F
    Ff = KeyboardUsage::KeyboardFf,
    /// Keyboard g and G
    Gg = KeyboardUsage::KeyboardGg,
    /// Keyboard h and H
    Hh = KeyboardUsage::KeyboardHh,
    /// Keyboard i and I
    Ii = KeyboardUsage::KeyboardIi,
    /// Keyboard j and J
    Jj = KeyboardUsage::KeyboardJj,
    /// Keyboard k and K
    Kk = KeyboardUsage::KeyboardKk,
    /// Keyboard l and L
    Ll = KeyboardUsage::KeyboardLl,
    /// Keyboard m and M (Footnote 2)
    Mm = KeyboardUsage::KeyboardMm,
    /// Keyboard n and N
    Nn = KeyboardUsage::KeyboardNn,
    /// Keyboard o and O (Footnote 2)
    Oo = KeyboardUsage::KeyboardOo,
    /// Keyboard p and P (Footnote 2)
    Pp = KeyboardUsage::KeyboardPp,
    /// Keyboard q and Q (Footnote 2)
    Qq = KeyboardUsage::KeyboardQq,
    /// Keyboard r and R
    Rr = KeyboardUsage::KeyboardRr,
    /// Keyboard s and S
    Ss = KeyboardUsage::KeyboardSs,
    /// Keyboard t and T
    Tt = KeyboardUsage::KeyboardTt,
    /// Keyboard u and U
    Uu = KeyboardUsage::KeyboardUu,
    /// Keyboard v and V
    Vv = KeyboardUsage::KeyboardVv,
    /// Keyboard w and W (Footnote 2)
    Ww = KeyboardUsage::KeyboardWw,
    /// Keyboard x and X (Footnote 2)
    Xx = KeyboardUsage::KeyboardXx,
    /// Keyboard y and Y (Footnote 2)
    Yy = KeyboardUsage::KeyboardYy,
    /// Keyboard z and Z (Footnote 2)
    Zz = KeyboardUsage::KeyboardZz,

    // ------------------------------------------------------------------------
    // 0x1E‑0x27: Number row (with shifted symbols)
    /// Keyboard 1 and ! (Footnote 2)
    K1 = KeyboardUsage::Keyboard1Exclamation,
    /// Keyboard 2 and @ (Footnote 2)
    K2 = KeyboardUsage::Keyboard2At,
    /// Keyboard 3 and # (Footnote 2)
    K3 = KeyboardUsage::Keyboard3Hash,
    /// Keyboard 4 and $ (Footnote 2)
    K4 = KeyboardUsage::Keyboard4Dollar,
    /// Keyboard 5 and % (Footnote 2)
    K5 = KeyboardUsage::Keyboard5Percent,
    /// Keyboard 6 and ^ (Footnote 2)
    K6 = KeyboardUsage::Keyboard6Caret,
    /// Keyboard 7 and & (Footnote 2)
    K7 = KeyboardUsage::Keyboard7Ampersand,
    /// Keyboard 8 and * (Footnote 2)
    K8 = KeyboardUsage::Keyboard8Asterisk,
    /// Keyboard 9 and ( (Footnote 2)
    K9 = KeyboardUsage::Keyboard9OpenParens,
    /// Keyboard 0 and ) (Footnote 2)
    K0 = KeyboardUsage::Keyboard0CloseParens,

    // ------------------------------------------------------------------------
    // 0x28‑0x2C: Basic control keys
    /// Keyboard Return (ENTER) (Footnote 3)
    Enter = KeyboardUsage::KeyboardEnter,
    /// Keyboard ESCAPE
    Escape = KeyboardUsage::KeyboardEscape,
    /// Keyboard DELETE (Backspace) (Footnote 4)
    Backspace = KeyboardUsage::KeyboardBackspace,
    /// Keyboard Tab
    Tab = KeyboardUsage::KeyboardTab,
    /// Keyboard Spacebar
    Space = KeyboardUsage::KeyboardSpacebar,

    // ------------------------------------------------------------------------
    // 0x2D‑0x35: Symbol keys
    /// Keyboard - and _ (Footnote 2)
    DashUnderscore = KeyboardUsage::KeyboardDashUnderscore,
    /// Keyboard = and + (Footnote 2)
    Equal = KeyboardUsage::KeyboardEqualPlus,
    /// Keyboard [ and { (Footnote 2)
    OpenBracket = KeyboardUsage::KeyboardOpenBracketBrace,
    /// Keyboard ] and } (Footnote 2)
    CloseBracket = KeyboardUsage::KeyboardCloseBracketBrace,
    /// Keyboard \ and |
    Bslash = KeyboardUsage::KeyboardBackslashBar,
    /// Keyboard Non‑US # (Footnote 5)
    NonUSHash = KeyboardUsage::KeyboardNonUSHash,
    /// Keyboard ; and : (Footnote 2)
    SemiColon = KeyboardUsage::KeyboardSemiColon,
    /// Keyboard ' and " (Footnote 2)
    Quote = KeyboardUsage::KeyboardSingleDoubleQuote,
    /// Keyboard ` and ~ (Footnote 2)
    BacktickTilde = KeyboardUsage::KeyboardBacktickTilde,
    /// Keyboard , and < (Footnote 2)
    Comma = KeyboardUsage::KeyboardCommaLess,
    /// Keyboard . and > (Footnote 2)
    Period = KeyboardUsage::KeyboardPeriodGreater,
    /// Keyboard / and ? (Footnote 2)
    Fslash = KeyboardUsage::KeyboardSlashQuestion,
    /// Keyboard Caps Lock (Footnote 6)
    CapsLock = KeyboardUsage::KeyboardCapsLock,

    // ------------------------------------------------------------------------
    // 0x3A‑0x45: Function keys
    F1 = KeyboardUsage::KeyboardF1,
    F2 = KeyboardUsage::KeyboardF2,
    F3 = KeyboardUsage::KeyboardF3,
    F4 = KeyboardUsage::KeyboardF4,
    F5 = KeyboardUsage::KeyboardF5,
    F6 = KeyboardUsage::KeyboardF6,
    F7 = KeyboardUsage::KeyboardF7,
    F8 = KeyboardUsage::KeyboardF8,
    F9 = KeyboardUsage::KeyboardF9,
    F10 = KeyboardUsage::KeyboardF10,
    F11 = KeyboardUsage::KeyboardF11,
    F12 = KeyboardUsage::KeyboardF12,
    F13 = KeyboardUsage::KeyboardF13,
    F14 = KeyboardUsage::KeyboardF14,
    F15 = KeyboardUsage::KeyboardF15,
    F16 = KeyboardUsage::KeyboardF16,
    F17 = KeyboardUsage::KeyboardF17,
    F18 = KeyboardUsage::KeyboardF18,
    F19 = KeyboardUsage::KeyboardF19,
    F20 = KeyboardUsage::KeyboardF20,
    F21 = KeyboardUsage::KeyboardF21,
    F22 = KeyboardUsage::KeyboardF22,
    F23 = KeyboardUsage::KeyboardF23,
    F24 = KeyboardUsage::KeyboardF24,

    // ------------------------------------------------------------------------
    // 0x46‑0x52: System / navigation keys
    /// Keyboard PrintScreen (Footnote 7)
    PrintS = KeyboardUsage::KeyboardPrintScreen,
    /// Keyboard ScrollLock (Footnote 6)
    ScrollLock = KeyboardUsage::KeyboardScrollLock,
    /// Keyboard Pause (Footnote 7)
    Pause = KeyboardUsage::KeyboardPause,
    /// Keyboard Insert (Footnote 7)
    Insert = KeyboardUsage::KeyboardInsert,
    /// Keyboard Home (Footnote 7)
    Home = KeyboardUsage::KeyboardHome,
    /// Keyboard PageUp (Footnote 7)
    PageUp = KeyboardUsage::KeyboardPageUp,
    /// Keyboard Delete Forward (Footnote 7, 8)
    Delete = KeyboardUsage::KeyboardDelete,
    /// Keyboard End (Footnote 7)
    End = KeyboardUsage::KeyboardEnd,
    /// Keyboard PageDown (Footnote 7)
    PageDown = KeyboardUsage::KeyboardPageDown,
    /// Keyboard RightArrow (Footnote 7)
    RightArr = KeyboardUsage::KeyboardRightArrow,
    /// Keyboard LeftArrow (Footnote 7)
    LeftArr = KeyboardUsage::KeyboardLeftArrow,
    /// Keyboard DownArrow (Footnote 7)
    DownArr = KeyboardUsage::KeyboardDownArrow,
    /// Keyboard UpArrow (Footnote 7)
    UpArr = KeyboardUsage::KeyboardUpArrow,

    // ------------------------------------------------------------------------
    // 0x53‑0x58: Keypad basics
    /// Keypad Num Lock and Clear (Footnote 6)
    NumLock = KeyboardUsage::KeypadNumLock,
    /// Keypad / (Footnote 7)
    KeypadDivide = KeyboardUsage::KeypadDivide,
    /// Keypad *
    KeypadMultiply = KeyboardUsage::KeypadMultiply,
    /// Keypad -
    KMinus = KeyboardUsage::KeypadMinus,
    /// Keypad +
    KeypadPlus = KeyboardUsage::KeypadPlus,
    /// Keypad ENTER (Footnote 3)
    KeypadEnter = KeyboardUsage::KeypadEnter,

    // ------------------------------------------------------------------------
    // 0x59‑0x63: Keypad extended keys
    /// Keypad 1 and End
    Keypad1End = KeyboardUsage::Keypad1End,
    /// Keypad 2 and DownArrow
    Keypad2DownArrow = KeyboardUsage::Keypad2DownArrow,
    /// Keypad 3 and PageDown
    Keypad3PageDown = KeyboardUsage::Keyboard3PageDown,
    /// Keypad 4 and LeftArrow
    Keypad4LeftArrow = KeyboardUsage::Keypad4LeftArrow,
    /// Keypad 5
    Keypad5 = KeyboardUsage::Keypad5,
    /// Keypad 6 and RightArrow
    Keypad6RightArrow = KeyboardUsage::Keypad6RightArrow,
    /// Keypad 7 and Home
    Keypad7Home = KeyboardUsage::Keypad7Home,
    /// Keypad 8 and UpArrow
    Keypad8UpArrow = KeyboardUsage::Keypad8UpArrow,
    /// Keypad 9 and PageUp
    Keypad9PageUp = KeyboardUsage::Keypad9PageUp,
    /// Keypad 0 and Insert
    Keypad0Insert = KeyboardUsage::Keypad0Insert,
    /// Keypad . and Delete
    KeypadPeriodDelete = KeyboardUsage::KeypadPeriodDelete,

    // ------------------------------------------------------------------------
    // 0x64‑0x65: Miscellaneous keys
    /// Keyboard Non‑US \ and | (Footnote 9, 10)
    USSlash = KeyboardUsage::KeyboardNonUSSlash,
    /// Keyboard Application (Footnote 11)
    Application = KeyboardUsage::KeyboardApplication,
    /// Keyboard Power (Footnote 1)
    Power = KeyboardUsage::KeyboardPower,

    // ------------------------------------------------------------------------
    // 0x66‑0x67: Keypad extra
    /// Keypad =
    KeypadEqual = KeyboardUsage::KeypadEqual,

    // ------------------------------------------------------------------------
    // 0x68‑0x73: Additional function keys
    F13 = KeyboardUsage::KeyboardF13,
    F14 = KeyboardUsage::KeyboardF14,
    F15 = KeyboardUsage::KeyboardF15,
    F16 = KeyboardUsage::KeyboardF16,
    F17 = KeyboardUsage::KeyboardF17,
    F18 = KeyboardUsage::KeyboardF18,
    F19 = KeyboardUsage::KeyboardF19,
    F20 = KeyboardUsage::KeyboardF20,
    F21 = KeyboardUsage::KeyboardF21,
    F22 = KeyboardUsage::KeyboardF22,
    F23 = KeyboardUsage::KeyboardF23,
    F24 = KeyboardUsage::KeyboardF24,

    // ------------------------------------------------------------------------
    // 0x74‑0x7D: System control keys
    Execute = KeyboardUsage::KeyboardExecute,
    Help = KeyboardUsage::KeyboardHelp,
    Menu = KeyboardUsage::KeyboardMenu,
    Select = KeyboardUsage::KeyboardSelect,
    Stop = KeyboardUsage::KeyboardStop,
    Again = KeyboardUsage::KeyboardAgain,
    Undo = KeyboardUsage::KeyboardUndo,
    Cut = KeyboardUsage::KeyboardCut,
    Copy = KeyboardUsage::KeyboardCopy,
    Paste = KeyboardUsage::KeyboardPaste,
    Find = KeyboardUsage::KeyboardFind,
    Mute = KeyboardUsage::KeyboardMute,
    VolumeUp = KeyboardUsage::KeyboardVolumeUp,
    VolumeDown = KeyboardUsage::KeyboardVolumeDown,

    // ------------------------------------------------------------------------
    // 0x7E‑0x84: Locking keys
    LockingCapsLock = KeyboardUsage::KeyboardLockingCapsLock,
    LockingNumLock = KeyboardUsage::KeyboardLockingNumLock,
    LockingScrollLock = KeyboardUsage::KeyboardLockingScrollLock,

    // ------------------------------------------------------------------------
    // 0x85‑0x86: Keypad punctuation
    KeypadComma = KeyboardUsage::KeypadComma,
    KeypadEqualSign = KeyboardUsage::KeypadEqualSign,

    // ------------------------------------------------------------------------
    // 0x87‑0x8F: International keys
    International1 = KeyboardUsage::KeyboardInternational1,
    International2 = KeyboardUsage::KeyboardInternational2,
    International3 = KeyboardUsage::KeyboardInternational3,
    International4 = KeyboardUsage::KeyboardInternational4,
    International5 = KeyboardUsage::KeyboardInternational5,
    International6 = KeyboardUsage::KeyboardInternational6,
    International7 = KeyboardUsage::KeyboardInternational7,
    International8 = KeyboardUsage::KeyboardInternational8,
    International9 = KeyboardUsage::KeyboardInternational9,

    // ------------------------------------------------------------------------
    // 0x90‑0x98: Language keys
    LANG1 = KeyboardUsage::KeyboardLANG1,
    LANG2 = KeyboardUsage::KeyboardLANG2,
    LANG3 = KeyboardUsage::KeyboardLANG3,
    LANG4 = KeyboardUsage::KeyboardLANG4,
    LANG5 = KeyboardUsage::KeyboardLANG5,
    LANG6 = KeyboardUsage::KeyboardLANG6,
    LANG7 = KeyboardUsage::KeyboardLANG7,
    LANG8 = KeyboardUsage::KeyboardLANG8,
    LANG9 = KeyboardUsage::KeyboardLANG9,

    // ------------------------------------------------------------------------
    // 0x99‑0x9C: Misc system keys
    AlternateErase = KeyboardUsage::KeyboardAlternateErase,
    SysReqAttention = KeyboardUsage::KeyboardSysReqAttention,
    Cancel = KeyboardUsage::KeyboardCancel,
    Clear = KeyboardUsage::KeyboardClear,

    // ------------------------------------------------------------------------
    // 0x9D‑0xA4: Navigation / selection keys
    Prior = KeyboardUsage::KeyboardPrior,
    Return = KeyboardUsage::KeyboardReturn,
    Separator = KeyboardUsage::KeyboardSeparator,
    Out = KeyboardUsage::KeyboardOut,
    Oper = KeyboardUsage::KeyboardOper,
    ClearAgain = KeyboardUsage::KeyboardClearAgain,
    CrSelProps = KeyboardUsage::KeyboardCrSelProps,
    ExSel = KeyboardUsage::KeyboardExSel,

    // ------------------------------------------------------------------------
    // 0xB0‑0xBF: Keypad numeric extensions
    Keypad00 = KeyboardUsage::Keypad00,
    Keypad000 = KeyboardUsage::Keypad000,
    ThousandsSeparator = KeyboardUsage::ThousandsSeparator,
    DecimalSeparator = KeyboardUsage::DecimalSeparator,
    CurrencyUnit = KeyboardUsage::CurrencyUnit,
    CurrencySubunit = KeyboardUsage::CurrencySubunit,
    OpenParens = KeyboardUsage::KeypadOpenParens,
    CloseParens = KeyboardUsage::KeypadCloseParens,
    OpenBrace = KeyboardUsage::KeypadOpenBrace,
    CloseBrace = KeyboardUsage::KeypadCloseBrace,
    Tab = KeyboardUsage::KeypadTab,
    Backspace = KeyboardUsage::KeypadBackspace,
    A = KeyboardUsage::KeypadA,
    B = KeyboardUsage::KeypadB,
    C = KeyboardUsage::KeypadC,
    D = KeyboardUsage::KeypadD,

    // ------------------------------------------------------------------------
    // 0xC0‑0xCA: Keypad logical / bitwise ops
    E = KeyboardUsage::KeypadE,
    F = KeyboardUsage::KeypadF,
    BitwiseXor = KeyboardUsage::KeypadBitwiseXor,
    LogicalXor = KeyboardUsage::KeypadLogicalXor,
    Modulo = KeyboardUsage::KeypadModulo,
    LShift = KeyboardUsage::KeypadLeftShift,
    RightShift = KeyboardUsage::KeypadRightShift,
    BitwiseAnd = KeyboardUsage::KeypadBitwiseAnd,
    LogicalAnd = KeyboardUsage::KeypadLogicalAnd,
    BitwiseOr = KeyboardUsage::KeypadBitwiseOr,
    LogicalOr = KeyboardUsage::KeypadLogicalOr,
    Colon = KeyboardUsage::KeypadColon,
    Hash = KeyboardUsage::KeypadHash,
    Space = KeyboardUsage::KeypadSpace,
    At = KeyboardUsage::KeypadAt,
    Exclamation = KeyboardUsage::KeypadExclamation,

    // ------------------------------------------------------------------------
    // 0xD0‑0xD9: Keypad memory functions
    MemoryStore = KeyboardUsage::KeypadMemoryStore,
    MemoryRecall = KeyboardUsage::KeypadMemoryRecall,
    MemoryClear = KeyboardUsage::KeypadMemoryClear,
    MemoryAdd = KeyboardUsage::KeypadMemoryAdd,

    MemorySubtract = KeyboardUsage::KeypadMemorySubtract,
    MemoryMultiply = KeyboardUsage::KeypadMemoryMultiply,
    MemoryDivide = KeyboardUsage::KeypadMemoryDivide,
    PositiveNegative = KeyboardUsage::KeypadPositiveNegative,
    Clear = KeyboardUsage::KeypadClear,
    ClearEntry = KeyboardUsage::KeypadClearEntry,
    Binary = KeyboardUsage::KeypadBinary,
    Octal = KeyboardUsage::KeypadOctal,
    Decimal = KeyboardUsage::KeypadDecimal,
    Hexadecimal = KeyboardUsage::KeypadHexadecimal,

    // ------------------------------------------------------------------------
    // 0xE0‑0xE7: Modifier keys
    LCtrl = KeyboardUsage::KeyboardLeftControl,
    LeftShift = KeyboardUsage::KeyboardLeftShift,
    LAlt = KeyboardUsage::KeyboardLeftAlt,
    LGUI = KeyboardUsage::KeyboardLeftGUI,
    RCtrs = KeyboardUsage::KeyboardRightControl,
    RShift = KeyboardUsage::KeyboardRightShift,
    RAlt = KeyboardUsage::KeyboardRightAlt,
    RGUI = KeyboardUsage::KeyboardRightGUI,

    // ------------------------------------------------------------------------
    // 0xE8‑0xFF: Reserved / invalid values
    Reserved = KeyboardUsage::Reserved,

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
