use std::fmt;

#[derive(Clone, Copy)]
pub enum BurnTime {
    Infinity,
    Time(f32),
}

impl BurnTime {
    pub fn inf_eq(&self, b2: &BurnTime) -> bool {
        // return true if self <= b2
        match (self, b2) {
            (BurnTime::Infinity, BurnTime::Infinity) => true,
            (BurnTime::Infinity, BurnTime::Time(_)) => false,
            (BurnTime::Time(_), BurnTime::Infinity) => true,
            (BurnTime::Time(f1), BurnTime::Time(f2)) => f1 <= f2,
        }
    }
}

impl fmt::Display for BurnTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BurnTime::Infinity => write!(f, "Infinity"),
            BurnTime::Time(t) => write!(f, "Time({})", t),
        }
    }
}
