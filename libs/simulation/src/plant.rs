use crate::*;

#[derive(Debug)]
pub struct Plant{
    pub(crate) position: na::Point2<f32>,
    pub(crate) eaten: bool,
}
impl Plant {
    pub fn random(rng: &mut dyn rand::RngCore) -> Self {
        Self {
            position: rng.gen(),
            eaten: false
        }
    }

    pub fn eaten(&self) -> bool {
        return self.eaten;
    }

    pub fn position(&self) -> na::Point2<f32> {
        return self.position;
    }
}