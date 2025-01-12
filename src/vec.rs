#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance(&self, other: &Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn add(&self, other: &Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn add_scalar(&self, other: f32) -> Self {
        Self {
            x: self.x + other,
            y: self.y + other,
        }
    }

    pub fn sub(&self, other: &Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn div(&self, val: f32) -> Self {
        assert!(val > 0., "Divider must be positive! Got {}", val);
        Self {
            x: self.x / val,
            y: self.y / val,
        }
    }

    pub fn mul(&self, val: f32) -> Self {
        Self {
            x: self.x * val,
            y: self.y * val,
        }
    }

    /// Turn into unit vector
    pub fn normalise(&self) -> Self {
        let magnitude = (self.x.powi(2) + self.y.powi(2)).sqrt();
        if magnitude == 0. {
            Self { x: 0., y: 0. }
        } else {
            Self {
                x: self.x / magnitude,
                y: self.y / magnitude,
            }
        }
    }

    pub fn direction_radians(&self) -> f32 {
        self.y.atan2(self.x)
    }
}
