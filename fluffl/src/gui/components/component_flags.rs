use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

/// the component has been flagged as the "titlebar"
pub const TITLEBAR: ComponentFlags = ComponentFlags(1 << 0);

/// the component is allowed to overflow that parent
pub const OVERFLOWABLE: ComponentFlags = ComponentFlags(1 << 1);

/// the component is marked as "visible"
pub const VISIBLE: ComponentFlags = ComponentFlags(1 << 2);

#[derive(Default, Copy, Clone)]
pub struct ComponentFlags(u64);

impl ComponentFlags {
    pub fn as_mask(mask: bool) -> ComponentFlags {
        Self((!0) * (mask as u64))
    }

    pub fn is_set(self, f: Self) -> bool {
        (self & f).to_bits() != 0
    }

    pub fn set(&mut self, flags: Self) {
        self.0 |= flags.to_bits();
    }

    pub fn unset(&mut self, flags: Self) {
        self.0 &= !flags.to_bits();
    }

    fn to_bits(self) -> u64 {
        self.0
    }
}

impl BitOr for ComponentFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for ComponentFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for ComponentFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for ComponentFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}
