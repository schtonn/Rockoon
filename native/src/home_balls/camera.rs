use super::math::V;
#[derive(Clone, Copy)]
pub struct Camera {
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    right: V,
    up: V,
    depth: V,
}
impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        let (s, c) = (-24.0_f32).to_radians().sin_cos();
        let (sp, cp) = 32.0_f32.to_radians().sin_cos();
        Self {
            width,
            height,
            scale: (width / 62.0).clamp(10.0, 16.0),
            right: V(c, 0.0, -s),
            up: V(-s * sp, cp, -c * sp),
            depth: V(s * cp, sp, c * cp),
        }
    }
    pub fn vector(self, p: V) -> V {
        V(p.dot(self.right), p.dot(self.up), p.dot(self.depth))
    }
    pub fn project(self, p: V) -> V {
        let p = self.vector(p);
        let f = 90.0 / (90.0 - p.2).max(10.0);
        V(
            self.width * 0.5 + p.0 * self.scale * f,
            self.height * 0.60 - p.1 * self.scale * f,
            p.2,
        )
    }
    pub fn factor(self, depth: f32) -> f32 {
        90.0 / (90.0 - depth).max(10.0)
    }
    pub fn ceiling(self, x: f32, z: f32) -> f32 {
        let target = (self.height * 0.60 + 60.0) / self.scale;
        let base = V(x, 0.0, z);
        let u = base.dot(self.up);
        let d = base.dot(self.depth);
        (target * (90.0 - d) - 90.0 * u) / (90.0 * self.up.1 + target * self.depth.1)
    }
    pub fn ground(self, x: f32, y: f32) -> V {
        let x = (x - self.width * 0.5) / self.scale;
        let y = (self.height * 0.60 - y) / self.scale;
        let a = 90.0 * self.right.0 + x * self.depth.0;
        let b = 90.0 * self.right.2 + x * self.depth.2;
        let c = 90.0 * self.up.0 + y * self.depth.0;
        let d = 90.0 * self.up.2 + y * self.depth.2;
        let det = a * d - b * c;
        V(
            (90.0 * x * d - b * 90.0 * y) / det,
            0.0,
            (a * 90.0 * y - 90.0 * x * c) / det,
        )
    }
    pub fn visible(self, p: V) -> bool {
        let s = self.project(p);
        s.0 > -160.0 && s.0 < self.width + 160.0 && s.1 < self.height + 160.0 && p.2.abs() < 65.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn floor_projection_roundtrips_and_spawn_is_above_window() {
        for (w, h) in [(1040.0, 700.0), (840.0, 580.0)] {
            let camera = Camera::new(w, h);
            for p in [V(0.0, 0.0, 0.0), V(8.0, 0.0, 4.0), V(-8.0, 0.0, -4.0)] {
                let screen = camera.project(p);
                assert!((camera.ground(screen.0, screen.1) - p).length() < 0.001);
                let spawn = V(p.0, camera.ceiling(p.0, p.2), p.2);
                assert!(camera.project(spawn).1 < -59.9);
            }
        }
    }
}
