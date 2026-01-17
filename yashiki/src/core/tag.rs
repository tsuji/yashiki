#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tag(u32);

impl Tag {
    pub fn new(n: u32) -> Self {
        assert!(n > 0 && n <= 32);
        Self(1 << (n - 1))
    }

    pub fn from_mask(mask: u32) -> Self {
        Self(mask)
    }

    pub fn mask(self) -> u32 {
        self.0
    }

    pub fn intersects(self, other: Tag) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn union(self, other: Tag) -> Self {
        Self(self.0 | other.0)
    }

    pub fn toggle(self, other: Tag) -> Self {
        Self(self.0 ^ other.0)
    }

    /// Returns the tag number (1-32) of the lowest set bit, or None if empty
    pub fn first_tag(self) -> Option<u32> {
        if self.0 == 0 {
            return None;
        }
        Some(self.0.trailing_zeros() + 1)
    }
}
