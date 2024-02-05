use std::f64;

#[derive(Debug, Copy, Clone )]
pub struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
    pub const ZERO: Vector = Vector { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Vector {
        Vector { x, y }
    }

    pub fn from_points(a: Vector, b: Vector) -> Vector {
        Vector {
            x: b.x - a.x,
            y: b.y - a.y,
        }
    }

    pub fn sqr_euclidean_to(&self, other: Vector) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    pub fn from_angle(angle: f64) -> Vector {
        Vector {
            x: angle.cos(),
            y: angle.sin(),
        }
    }

    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }

    pub fn hsymmetric(&self, center: f64) -> Vector {
        Vector {
            x: 2.0 * center - self.x,
            y: self.y,
        }
    }

    pub fn vsymmetric(&self, center: f64) -> Vector {
        Vector {
            x: self.x,
            y: 2.0 * center - self.y,
        }
    }

    pub fn epsilon_round(&self) -> Vector {
        Vector {
            x: (self.x * 1e7).round() / 1e7,
            y: (self.y * 1e7).round() / 1e7,
        }
    }

    pub fn rotate(&self, angle: f64) -> Vector {
        let nx = self.x * angle.cos() - self.y * angle.sin();
        let ny = self.x * angle.sin() + self.y * angle.cos();

        Vector { x: nx, y: ny }
    }

    pub fn equals(&self, other: Vector) -> bool {
        self.x == other.x && self.y == other.y
    }

    pub fn round(&self) -> Vector {
        Vector {
            x: self.x.round(),
            y: self.y.round(),
        }
    }

    pub fn truncate(&self) -> Vector {
        Vector {
            x: self.x.trunc(),
            y: self.y.trunc(),
        }
    }

    pub fn distance(&self, v: Vector) -> f64 {
        ((v.x - self.x).powi(2) + (v.y - self.y).powi(2)).sqrt()
    }

    pub fn in_range(&self, v: &Vector, range: f64) -> bool {
        (v.x - self.x).powi(2) + (v.y - self.y).powi(2) <= range.powi(2)
    }

    pub fn add(&self, v: Vector) -> Vector {
        Vector {
            x: self.x + v.x,
            y: self.y + v.y,
        }
    }

    pub fn mult(&self, factor: f64) -> Vector {
        Vector {
            x: self.x * factor,
            y: self.y * factor,
        }
    }

    pub fn sub(&self, v: Vector) -> Vector {
        Vector {
            x: self.x - v.x,
            y: self.y - v.y,
        }
    }

    pub fn length(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn length_squared(&self) -> f64 {
        self.x.powi(2) + self.y.powi(2)
    }

    pub fn normalize(&self) -> Vector {
        let length = self.length();
        if length == 0.0 {
            Vector::ZERO
        } else {
            Vector {
                x: self.x / length,
                y: self.y / length,
            }
        }
    }

    pub fn dot(&self, v: Vector) -> f64 {
        self.x * v.x + self.y * v.y
    }

    pub fn angle(&self) -> f64 {
        self.y.atan2(self.x)
    }

    pub fn to_string(&self) -> String {
        format!("[{}, {}]", self.x, self.y)
    }

    // Add other methods here...
}