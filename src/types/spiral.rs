use crate::types::point::Point;

pub(crate) struct Spiral {
    b: f64,

    theta: f64,
}

impl Spiral {
    pub(crate) fn new(b: f64) -> Spiral {
        Spiral { b, theta: 0. }
    }

    pub(crate) fn advance(&mut self) {
        let revelations = self.theta / (std::f64::consts::PI * 2.0f64);
        if revelations < 5.0 {
            self.theta += 1.0;
        } else {
            self.theta += 0.1_f64;
        }
    }

    pub(crate) fn position(&self) -> Point<f32> {
        let r = self.b * self.theta;

        Point {
            x: (r * self.theta.cos()) as f32,
            y: (r * self.theta.sin()) as f32,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.theta = 0.;
    }
}
