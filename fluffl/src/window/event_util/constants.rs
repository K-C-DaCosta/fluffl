use crate::math::Vec2;
use serde::{Deserialize, Serialize};
use std::fmt;

pub const KP_OFFSET: isize = 1000;

//the whole point of this module is to provide a generic interface for events in the code.
//Every target needs to map its native events to these.
#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum EventKind {
    /// # Description
    /// This event fires only when the user clicks on the "x" button on desktop.
    /// This event doesn't really apply in the browser environment, unless I do my own virtual window thingy, which im
    /// not going to do
    Quit,

    /// # Description
    /// Whenever the window resizes this is called
    Resize {
        width: i32,
        height: i32,
    },

    /// # Description
    /// If the user moves the mouse, this event gets enqueued
    MouseMove {
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
    },

    /// # Description
    /// If the user pushes a mouse button, this event gets enqueued
    MouseDown {
        button_code: MouseCode,
        x: f32,
        y: f32,
    },

    /// # Description
    /// if the user releases a mouse button, this event gets enqueues
    /// # Members
    /// - `x` and `y` are absolute coordinates in standard screen space,
    /// so (0,0) is top-left corner and (width,height) is botton right corner of the window
    MouseUp {
        button_code: MouseCode,
        x: f32,
        y: f32,
    },

    /// # Description
    /// This event will appear in the event queue when something happens with the mouse wheel
    MouseWheel {
        button_code: MouseCode,
    },

    /// # Description
    /// This event should fire when a the underlying backend detects finger movement
    /// # Members
    /// - `finger_id` - a unique id given to each finger moving
    /// - `x`/`y` - the normalized absolute postions  
    TouchMove {
        finger_id: i32,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
    },

    TouchDown {
        finger_id: i32,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
    },

    TouchUp {
        finger_id: i32,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
    },

    KeyDown {
        code: KeyCode,
    },

    KeyUp {
        code: KeyCode,
    },
}
impl EventKind {
    pub fn mouse_pos(&self) -> Vec2<f32> {
        match *self {
            Self::MouseDown { x, y, .. } => Vec2::from([x, y]),
            Self::MouseUp { x, y, .. } => Vec2::from([x, y]),
            Self::MouseMove { x, y, .. } => Vec2::from([x, y]),
            Self::TouchMove { x, y, .. } => Vec2::from([x, y]),
            _ => Vec2::zero(),
        }
    }

    pub fn disp(&self) -> Vec2<f32> {
        match *self {
            Self::MouseMove { dx, dy, .. } => Vec2::from([dx, dy]),
            Self::TouchMove { dx, dy, .. } => Vec2::from([dx, dy]),
            _ => Vec2::zero(),
        }
    }

    pub fn wheel(&self) -> f32 {
        match self {
            &Self::MouseWheel {
                button_code: MouseCode::WHEEL { direction },
            } => direction as f32,
            _ => 0.0,
        }
    }

    pub fn disp_mouse_only(&self) -> Vec2<f32> {
        match self {
            &Self::MouseMove { dx, dy, .. } => Vec2::from([dx, dy]),
            _ => Vec2::zero(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, Debug)]
#[allow(non_camel_case_types)]
pub enum KeyCode {
    KEY_A = 'a' as isize,
    KEY_B = 'b' as isize,
    KEY_C = 'c' as isize,
    KEY_D = 'd' as isize,
    KEY_E = 'e' as isize,
    KEY_F = 'f' as isize,
    KEY_G = 'g' as isize,
    KEY_H = 'h' as isize,
    KEY_I = 'i' as isize,
    KEY_J = 'j' as isize,
    KEY_K = 'k' as isize,
    KEY_L = 'l' as isize,
    KEY_M = 'm' as isize,
    KEY_N = 'n' as isize,
    KEY_O = 'o' as isize,
    KEY_P = 'p' as isize,
    KEY_Q = 'q' as isize,
    KEY_R = 'r' as isize,
    KEY_S = 's' as isize,
    KEY_T = 't' as isize,
    KEY_U = 'u' as isize,
    KEY_V = 'v' as isize,
    KEY_W = 'w' as isize,
    KEY_X = 'x' as isize,
    KEY_Y = 'y' as isize,
    KEY_Z = 'z' as isize,
    NUM_0 = ('0' as isize),
    NUM_1 = ('1' as isize),
    NUM_2 = ('2' as isize),
    NUM_3 = ('3' as isize),
    NUM_4 = ('4' as isize),
    NUM_5 = ('5' as isize),
    NUM_6 = ('6' as isize),
    NUM_7 = ('7' as isize),
    NUM_8 = ('8' as isize),
    NUM_9 = ('9' as isize),
    MINUS = '-' as isize,
    EQUALS = '=' as isize,
    BRACKET_L = '{' as isize,
    BRACKET_R = '}' as isize,
    COLON = ';' as isize,
    SPACE = ' ' as isize,
    TAB = '\t' as isize,
    BACK_QUOTE = '`' as isize,
    QUOTE = '\'' as isize,
    BACKSLASH = '\\' as isize,
    COMMA = ',' as isize,
    PERIOD = '.' as isize,
    FORDSLASH = '/' as isize,
    ENTER = '\n' as isize,
    ARROW_L = 128,
    ARROW_R = 129,
    ARROW_U,
    ARROW_D,
    PAREN_RIGHT,
    PARENT_LEFT,
    HOME,
    INSERT,
    ALT_L,
    ALT_R,
    CTRL_L,
    CTRL_R,
    SHIFT_L,
    SHIFT_R,
    SUPER_L,
    MENU,
    NUMLOCK,
    PAUSE,
    PAGE_D,
    PAGE_U,
    POWER,
    PRINT_SCREEN,
    SUPER_R,
    SCROLL_LOCK,
    SLEEP,
    WAKE,
    BACKSPACE,
    CAPSLOCK,
    AT,
    DELETE,
    END,
    ESC,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    KP_0 = '0' as isize + KP_OFFSET,
    KP_1 = '1' as isize + KP_OFFSET,
    KP_2 = '2' as isize + KP_OFFSET,
    KP_3 = '3' as isize + KP_OFFSET,
    KP_4 = '4' as isize + KP_OFFSET,
    KP_5 = '5' as isize + KP_OFFSET,
    KP_6 = '6' as isize + KP_OFFSET,
    KP_7 = '7' as isize + KP_OFFSET,
    KP_8 = '8' as isize + KP_OFFSET,
    KP_9 = '9' as isize + KP_OFFSET,
    KP_STAR = '*' as isize + KP_OFFSET,
    KP_ENTER = '\n' as isize + KP_OFFSET,
    KP_INS,
    KP_END,
    KP_ARROW_D,
    KP_PAGE_D,
    KP_PLUS,
    KP_MINUS,
    KP_ARROW_L,
    KP_ARROW_R,
    KP_HOME,
    KP_ARROW_U,
    KP_PAGE_U,
    KP_DECIMAL,
    KP_DEL,
    KP_DASH,
    KP_FORDSLASH,
    UNKNOWN,
}

impl KeyCode {
    pub fn key_val(self) -> Option<char> {
        let code: i128 = self.into();
        if (code > KeyCode::KEY_A.into()) || (code < KeyCode::KEY_Z.into()) {
            let c = code as u8 as char;
            Some(c)
        } else {
            None
        }
    }
}

impl From<KeyCode> for i128 {
    fn from(a: KeyCode) -> Self {
        a as Self
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum MouseCode {
    LEFT_BUTTON,
    RIGHT_BUTTON,
    WHEEL { direction: i32 },
}

impl fmt::Display for MouseCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MouseCode::LEFT_BUTTON => writeln!(f, "left button"),
            MouseCode::RIGHT_BUTTON => writeln!(f, "right button"),
            MouseCode::WHEEL { direction } => writeln!(f, "wheel dir:{}", direction),
        }
    }
}
