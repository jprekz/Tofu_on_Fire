use amethyst::core::nalgebra::{Real, Vector2};

pub trait Vector2Ext<N> {
    fn to_polar(&self) -> (N, N);
    fn from_polar(r: N, theta: N) -> Self;
}
impl<N: Real> Vector2Ext<N> for Vector2<N> {
    fn to_polar(&self) -> (N, N) {
        (self.x.hypot(self.y), self.y.atan2(self.x))
    }
    fn from_polar(r: N, theta: N) -> Self {
        Vector2::new(theta.cos() * r, theta.sin() * r)
    }
}
