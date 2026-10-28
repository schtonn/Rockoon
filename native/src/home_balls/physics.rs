//! Decorative sphere-only approximation, using the exported Physicalize_GameBall profiles.
//! Gravity and fixed step come from the retail journal. This is not the IVP solver.
use super::{
    camera::Camera,
    math::{Q, V},
    mesh::Mesh,
};
use std::sync::Arc;
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
    pub platform: Option<Arc<Collider>>,
    pub bodies: Vec<Body>,
    seed: u64,
    pub(super) serial: usize,
    next: f32,
    remainder: f32,
}
impl World {
    pub fn new(seed: u64) -> Self {
        Self {
            platform: None,
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
    fn spawn(&mut self, camera: Camera) {
        let kind = [2, 0, 1][self.serial % 3];
        let target_x = (self.random() - 0.5) * 8.0;
        let target_z = (self.random() - 0.5) * 4.0;
        let angular = V(
            (self.random() - 0.5) * 6.0,
            (self.random() - 0.5) * 6.0,
            (self.random() - 0.5) * 6.0,
        );
        let mut rotation = Q::default();
        rotation.advance(angular, 8.0);
        let velocity = V(
            (self.random() - 0.5) * 16.0,
            -0.5 - self.random() * 3.0,
            (self.random() - 0.5) * 14.0,
        );
        // Stronger launch momentum, aimed loosely toward the finite platform after air drag.
        let damping = PROFILES[kind].damping;
        let travel = (1.0 - (-damping * 2.3).exp()) / damping * 0.85;
        let x = target_x - velocity.0 * travel;
        let z = target_z - velocity.2 * travel;
        self.bodies.push(Body {
            kind,
            position: V(x, camera.ceiling(x, z), z),
            velocity,
            angular,
            rotation,
        });
        self.serial += 1;
        self.next = 1.15 + self.random() * 0.65;
    }
    pub fn advance(&mut self, dt: f32, width: f32, height: f32) {
        let camera = Camera::new(width, height);
        // A resumed/blocked window never attempts to simulate minutes of missed frames.
        self.remainder += dt.clamp(0.0, 0.1);
        while self.remainder >= STEP {
            self.remainder -= STEP;
            self.next -= STEP;
            if self.next <= 0.0 && self.bodies.len() < 12 {
                self.spawn(camera);
            }
            for b in &mut self.bodies {
                b.integrate(STEP);
            }
            for _ in 0..3 {
                if let Some(platform) = &self.platform {
                    for b in &mut self.bodies {
                        platform.collide(b);
                    }
                }
                for i in 0..self.bodies.len() {
                    let (a, rest) = self.bodies.split_at_mut(i + 1);
                    for b in rest {
                        collide(&mut a[i], b);
                    }
                }
            }
            self.bodies.retain(|b| camera.visible(b.position));
        }
    }
    /// Bounded render prediction smooths the 66 Hz steps at the display's cadence.
    pub fn poses(&self) -> impl Iterator<Item = (usize, V, Q)> + '_ {
        self.bodies.iter().map(|body| {
            let mut pose = body.clone();
            pose.integrate(self.remainder);
            if let Some(platform) = &self.platform {
                platform.collide(&mut pose);
            }
            (pose.kind, pose.position, pose.rotation)
        })
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
    let contact =
        b.velocity + b.angular.cross(-n * RADIUS) - a.velocity - a.angular.cross(n * RADIUS);
    let slip = contact - n * contact.dot(n);
    if slip.length() < 1e-6 {
        return;
    }
    let tangent = slip.unit();
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

#[derive(Debug)]
pub struct Collider {
    triangles: Vec<[V; 3]>,
    min: V,
    max: V,
}
impl Collider {
    pub fn new(mesh: &Mesh) -> Self {
        let mut min = V(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = -min;
        for p in &mesh.positions {
            min = V(min.0.min(p.0), min.1.min(p.1), min.2.min(p.2));
            max = V(max.0.max(p.0), max.1.max(p.1), max.2.max(p.2));
        }
        Self {
            triangles: mesh
                .faces
                .iter()
                .map(|f| f.map(|i| mesh.positions[i]))
                .collect(),
            min,
            max,
        }
    }
    fn collide(&self, body: &mut Body) {
        let p = body.position;
        if p.0 < self.min.0 - RADIUS
            || p.0 > self.max.0 + RADIUS
            || p.1 < self.min.1 - RADIUS
            || p.1 > self.max.1 + RADIUS
            || p.2 < self.min.2 - RADIUS
            || p.2 > self.max.2 + RADIUS
        {
            return;
        }
        for &[a, b, c] in &self.triangles {
            let point = closest(body.position, a, b, c);
            let delta = body.position - point;
            let distance = delta.length();
            if distance >= RADIUS || distance < 1e-5 {
                continue;
            }
            let n = delta * (1.0 / distance);
            body.position = body.position + n * ((RADIUS - distance - 0.001).max(0.0));
            let speed = body.velocity.dot(n);
            if speed >= 0.0 {
                continue;
            }
            let profile = PROFILES[body.kind];
            // Retail A02_Floor_01 material: friction .7, elasticity .3.
            let restitution = if speed < -0.8 {
                (profile.elasticity * 0.3).sqrt()
            } else {
                0.0
            };
            let normal_impulse = -(1.0 + restitution) * speed * profile.mass;
            body.velocity = body.velocity + n * (normal_impulse / profile.mass);
            let contact = body.velocity + body.angular.cross(-n * RADIUS);
            let slip = contact - n * contact.dot(n);
            if slip.length() < 1e-6 {
                continue;
            }
            let tangent = slip.unit();
            let limit = (profile.friction * 0.7).sqrt() * normal_impulse;
            let impulse = tangent * ((-slip.length() * profile.mass / 3.5).max(-limit));
            body.velocity = body.velocity + impulse * (1.0 / profile.mass);
            body.angular = body.angular - n.cross(impulse) * (2.5 / (profile.mass * RADIUS));
        }
    }
}
// Closest point on a triangle, including vertices and edges (not an infinite plane).
fn closest(p: V, a: V, b: V, c: V) -> V {
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = p - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return a + ab * (d1 / (d1 - d3));
    }
    let cp = p - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return a + ac * (d2 / (d2 - d6));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        return b + (c - b) * ((d4 - d3) / ((d4 - d3) + (d5 - d6)));
    }
    let inv = (va + vb + vc).recip();
    a + ab * (vb * inv) + ac * (vc * inv)
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
            a.advance(1.0 / 60.0, 840.0, 700.0);
        }
        for _ in 0..300 {
            b.advance(1.0 / 30.0, 840.0, 700.0);
        }
        assert_eq!(a.bodies.len(), b.bodies.len());
        for (a, b) in a.bodies.iter().zip(&b.bodies) {
            assert!((a.position - b.position).length() < 0.001);
        }
        for _ in 0..3000 {
            a.advance(1.0 / 30.0, 840.0, 700.0);
            assert!(a.bodies.len() <= 12);
            assert!(
                a.bodies
                    .iter()
                    .all(|b| b.position.length().is_finite() && b.velocity.length() < 100.0)
            );
        }
        let mut render = World::new(7);
        render.advance(STEP, 840.0, 700.0);
        let before_pose = render.poses().next().unwrap().1;
        render.advance(STEP * 0.25, 840.0, 700.0);
        assert_eq!(render.bodies.len(), 1);
        let next_pose = render.poses().next().unwrap().1;
        assert!(next_pose.1 < before_pose.1); // Smooth movement between physics steps.
        let before = a.serial;
        a.advance(5000.0, 840.0, 700.0);
        assert!(a.serial - before <= 1);
    }
    #[test]
    fn finite_platform_bounces_supports_and_allows_falling_off() {
        let collider = Collider {
            triangles: vec![
                [
                    V(-10.0, 0.0, -10.0),
                    V(10.0, 0.0, -10.0),
                    V(10.0, 0.0, 10.0),
                ],
                [
                    V(-10.0, 0.0, -10.0),
                    V(10.0, 0.0, 10.0),
                    V(-10.0, 0.0, 10.0),
                ],
            ],
            min: V(-10.0, 0.0, -10.0),
            max: V(10.0, 0.0, 10.0),
        };
        let mut b = body(0, 0.0, 0.0);
        b.position = V(0.0, 1.9, 0.0);
        b.velocity = V(0.0, -8.0, 2.0);
        collider.collide(&mut b);
        assert!(b.velocity.1 > 0.0);
        assert!(b.position.1 > 1.99);
        for _ in 0..660 {
            b.integrate(STEP);
            collider.collide(&mut b);
        }
        assert!((b.position.1 - RADIUS).abs() < 0.01);
        b.position = V(0.0, 1.9, 14.0);
        b.velocity = V(0.0, -8.0, 0.0);
        collider.collide(&mut b);
        assert_eq!(b.velocity.1, -8.0);
        let mut a = body(1, 0.0, 0.0);
        let mut b = body(2, 0.0, 0.0);
        b.position.2 = 3.9;
        a.velocity.2 = 4.0;
        collide(&mut a, &mut b);
        assert!(b.velocity.2 > a.velocity.2);
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
