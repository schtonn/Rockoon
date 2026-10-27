use std::ops::{Add, Mul, Neg, Sub};
#[derive(Clone, Copy, Debug, Default)]
pub struct V(pub f32, pub f32, pub f32);
impl V {
    pub fn dot(self, b: Self) -> f32 {
        self.0 * b.0 + self.1 * b.1 + self.2 * b.2
    }
    pub fn cross(self, b: Self) -> Self {
        Self(
            self.1 * b.2 - self.2 * b.1,
            self.2 * b.0 - self.0 * b.2,
            self.0 * b.1 - self.1 * b.0,
        )
    }
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }
    pub fn unit(self) -> Self {
        self * (1.0 / self.length().max(1e-8))
    }
}
impl Add for V {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self(self.0 + b.0, self.1 + b.1, self.2 + b.2)
    }
}
impl Sub for V {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        self + -b
    }
}
impl Neg for V {
    type Output = Self;
    fn neg(self) -> Self {
        self * -1.0
    }
}
impl Mul<f32> for V {
    type Output = Self;
    fn mul(self, b: f32) -> Self {
        Self(self.0 * b, self.1 * b, self.2 * b)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Q(pub V, pub f32);
impl Default for Q {
    fn default() -> Self {
        Self(V::default(), 1.0)
    }
}
impl Q {
    pub fn rotate(self, v: V) -> V {
        let t = self.0.cross(v) * 2.0;
        v + t * self.1 + self.0.cross(t)
    }
    pub fn advance(&mut self, angular: V, dt: f32) {
        let speed = angular.length();
        if speed < 1e-7 {
            return;
        }
        let (s, c) = (speed * dt * 0.5).sin_cos();
        let v = angular * (s / speed);
        let xyz = v * self.1 + self.0 * c + v.cross(self.0);
        let w = c * self.1 - v.dot(self.0);
        let inverse = (xyz.dot(xyz) + w * w).sqrt().recip();
        *self = Self(xyz * inverse, w * inverse);
    }
}
