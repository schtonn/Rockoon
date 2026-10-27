//! Decorative sphere-only approximation, using the exported Physicalize_GameBall profiles.
//! Gravity and fixed step come from the retail journal. This is not the IVP solver.
use super::math::{Q, V};
pub const RADIUS: f32 = 2.0;
pub const STEP: f32 = 1.0 / 66.0;
#[derive(Clone, Copy)]
pub struct Profile {
    pub mass: f32,
    pub friction: f32,
    pub elasticity: f32,
    pub damping: f32,
}
pub const PROFILES: [Profile; 3] = [
    Profile {
        mass: 1.9,
        friction: 0.8,
        elasticity: 0.2,
        damping: 0.9,
    },
    Profile {
        mass: 10.0,
        friction: 0.5,
        elasticity: 0.1,
        damping: 0.3,
    },
    Profile {
        mass: 0.2,
        friction: 0.5,
        elasticity: 0.4,
        damping: 1.5,
    },
];
#[derive(Clone, Debug)]
pub struct Body {
    pub kind: usize,
    pub position: V,
    pub velocity: V,
    pub angular: V,
    pub rotation: Q,
}
impl Body {
    fn integrate(&mut self, dt: f32) {
        // Analytic constant gravity + linear damping avoids frame-rate-dependent fall speed.
        let damp = PROFILES[self.kind].damping;
        let decay = (-damp * dt).exp();
        let drift = (1.0 - decay) / damp;
        let terminal = V(0.0, -20.0 / damp, 0.0);
        self.position = self.position + terminal * dt + (self.velocity - terminal) * drift;
        self.velocity = terminal + (self.velocity - terminal) * decay;
        self.angular = self.angular * (-0.1 * dt).exp();
        self.rotation.advance(self.angular, dt);
    }
}
pub struct World {
    pub bodies: Vec<Body>,
    seed: u64,
    serial: usize,
    next: f32,
    remainder: f32,
}
impl World {
    pub fn new(seed: u64) -> Self {
        Self {
            bodies: Vec::new(),
            seed,
            serial: 0,
            next: 0.0,
            remainder: 0.0,
        }
    }
    fn random(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.seed >> 40) as f32 / (1u32 << 24) as f32
    }
    fn spawn(&mut self, width: f32, height: f32) {
        // A loose cluster occasionally lets a fast stone catch a lighter ball naturally.
        let kind = [2, 0, 1][self.serial % 3];
        let lane = [0.22, 0.51, 0.77][(self.serial / 3) % 3];
        let x = (width * (lane + (self.random() - 0.5) * 0.14)).clamp(3.0, (width - 3.0).max(3.0));
        let angular = V(
            (self.random() - 0.5) * 1.8,
            (self.random() - 0.5) * 2.4,
            (self.random() - 0.5) * 1.4,
        );
        let mut rotation = Q::default();
        rotation.advance(angular, 8.0);
        let velocity = V((self.random() - 0.5) * 3.0, -0.7, 0.0);
        self.bodies.push(Body {
            kind,
            position: V(x, height + RADIUS + 0.2, 0.0),
            velocity,
            angular,
            rotation,
        });
        self.serial += 1;
        self.next = 0.95 + self.random() * 0.4;
    }
    pub fn advance(&mut self, dt: f32, width: f32, height: f32) {
        // A resumed/blocked window never attempts to simulate minutes of missed frames.
        self.remainder += dt.clamp(0.0, 0.1);
        while self.remainder >= STEP {
            self.remainder -= STEP;
            self.next -= STEP;
            if self.next <= 0.0 && self.bodies.len() < 8 {
                self.spawn(width, height);
            }
            for b in &mut self.bodies {
                b.integrate(STEP);
            }
            for _ in 0..3 {
                for i in 0..self.bodies.len() {
                    let (a, rest) = self.bodies.split_at_mut(i + 1);
                    for b in rest {
                        collide(&mut a[i], b);
                    }
                }
            }
            self.bodies.retain(|b| {
                b.position.1 > -RADIUS * 2.0
                    && b.position.0 > -RADIUS * 2.0
                    && b.position.0 < width + RADIUS * 2.0
            });
        }
    }
    /// Bounded render prediction smooths the 66 Hz steps at the display's cadence.
    pub fn poses(&self) -> impl Iterator<Item = (usize, V, Q)> + '_ {
        self.bodies.iter().map(|body| {
            let mut pose = body.clone();
            pose.integrate(self.remainder);
            (pose.kind, pose.position, pose.rotation)
        })
    }
    pub fn resized(&mut self, sx: f32, sy: f32) {
        for b in &mut self.bodies {
            b.position.0 *= sx;
            b.position.1 *= sy;
        }
    }
}
fn collide(a: &mut Body, b: &mut Body) {
    let delta = b.position - a.position;
    let distance = delta.length();
    if distance >= RADIUS * 2.0 {
        return;
    }
    let n = if distance > 1e-6 {
        delta * (1.0 / distance)
    } else {
        V(1.0, 0.0, 0.0)
    };
    let pa = PROFILES[a.kind];
    let pb = PROFILES[b.kind];
    let ia = pa.mass.recip();
    let ib = pb.mass.recip();
    let inv = ia + ib;
    let correction = n * ((RADIUS * 2.0 - distance - 0.001).max(0.0) * 0.7 / inv);
    a.position = a.position - correction * ia;
    b.position = b.position + correction * ib;
    let relative = b.velocity - a.velocity;
    let approach = relative.dot(n);
    if approach >= 0.0 {
        return;
    }
    let impulse = -(1.0 + (pa.elasticity * pb.elasticity).sqrt()) * approach / inv;
    a.velocity = a.velocity - n * (impulse * ia);
    b.velocity = b.velocity + n * (impulse * ib);
    // Coulomb friction at the contact point transfers tangential movement into spin.
    let tangent = V(-n.1, n.0, 0.0);
    let contact =
        b.velocity + b.angular.cross(-n * RADIUS) - a.velocity - a.angular.cross(n * RADIUS);
    let inertia_a = ia * 2.5 / (RADIUS * RADIUS);
    let inertia_b = ib * 2.5 / (RADIUS * RADIUS);
    let friction = (pa.friction * pb.friction).sqrt() * impulse;
    let jt = (-contact.dot(tangent) / (inv + RADIUS * RADIUS * (inertia_a + inertia_b)))
        .clamp(-friction, friction);
    let j = tangent * jt;
    a.velocity = a.velocity - j * ia;
    b.velocity = b.velocity + j * ib;
    a.angular = a.angular - n.cross(j) * (RADIUS * inertia_a);
    b.angular = b.angular - n.cross(j) * (RADIUS * inertia_b);
}
#[cfg(test)]
mod tests {
    use super::*;
    fn body(kind: usize, x: f32, vx: f32) -> Body {
        Body {
            kind,
            position: V(x, 0.0, 0.0),
            velocity: V(vx, 0.0, 0.0),
            angular: V::default(),
            rotation: Q::default(),
        }
    }
    #[test]
    fn collision_conserves_momentum_and_loses_energy() {
        let mut a = body(1, 0.0, 4.0);
        let mut b = body(2, 3.9, 0.0);
        let momentum = 10.0 * a.velocity.0 + 0.2 * b.velocity.0;
        collide(&mut a, &mut b);
        assert!((10.0 * a.velocity.0 + 0.2 * b.velocity.0 - momentum).abs() < 1e-4);
        assert!(b.velocity.0 > a.velocity.0);
        assert!(10.0 * a.velocity.dot(a.velocity) + 0.2 * b.velocity.dot(b.velocity) < 160.0);
    }
    #[test]
    fn drag_distinguishes_balls_without_mass_dependent_gravity() {
        let mut wood = body(0, 0.0, 0.0);
        let mut stone = body(1, 0.0, 0.0);
        let mut paper = body(2, 0.0, 0.0);
        for _ in 0..66 {
            wood.integrate(STEP);
            stone.integrate(STEP);
            paper.integrate(STEP);
        }
        assert!(stone.position.1 < wood.position.1 && wood.position.1 < paper.position.1);
        assert!((paper.velocity.1 - (-20.0 / 1.5) * (1.0 - (-1.5_f32).exp())).abs() < 1e-4);
    }
    #[test]
    fn fixed_step_is_independent_of_display_rate_and_bounds_long_pauses() {
        let mut a = World::new(42);
        let mut b = World::new(42);
        for _ in 0..600 {
            a.advance(1.0 / 60.0, 60.0, 50.0);
        }
        for _ in 0..300 {
            b.advance(1.0 / 30.0, 60.0, 50.0);
        }
        assert_eq!(a.bodies.len(), b.bodies.len());
        for (a, b) in a.bodies.iter().zip(&b.bodies) {
            assert!((a.position - b.position).length() < 0.001);
        }
        for _ in 0..3000 {
            a.advance(1.0 / 30.0, 60.0, 50.0);
            assert!(a.bodies.len() <= 8);
            assert!(
                a.bodies
                    .iter()
                    .all(|b| b.position.length().is_finite() && b.velocity.length() < 100.0)
            );
        }
        let mut render = World::new(7);
        render.advance(STEP, 60.0, 50.0);
        let before_pose = render.poses().next().unwrap().1;
        render.advance(STEP * 0.25, 60.0, 50.0);
        assert_eq!(render.bodies.len(), 1);
        let next_pose = render.poses().next().unwrap().1;
        assert!(next_pose.1 < before_pose.1); // Smooth movement between physics steps.
        let before = a.serial;
        a.advance(5000.0, 60.0, 50.0);
        assert!(a.serial - before <= 1);
    }
    #[test]
    fn glancing_contact_generates_spin() {
        let mut a = body(0, 0.0, 4.0);
        let mut b = body(1, 3.5, 0.0);
        b.position.1 = 1.0;
        collide(&mut a, &mut b);
        assert!(a.angular.2.abs() > 0.01 && b.angular.2.abs() > 0.001);
    }
}
