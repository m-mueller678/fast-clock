use crate::{CalibratedClock, Clock};

#[derive(Clone, Copy)]
pub struct ClockSynchronization<A: Clock, B: Clock> {
    at: A::Instant,
    bt: B::Instant,
    a: A,
    b: B,
}

impl<A: CalibratedClock, B: Clock> ClockSynchronization<A, B> {
    pub fn new_aba(a: A, b: B) -> Self {
        let (a0, bt, da) = (0..3)
            .map(|_| {
                let a0 = a.now();
                let b = b.now();
                let a1 = a.now();
                let da = a.sub_i64_ns(a1, a0);
                (a0, b, da)
            })
            .min_by_key(|(.., da)| *da)
            .unwrap();
        ClockSynchronization {
            a,
            b,
            bt,
            at: a.add_i64_ns(a0, da / 2),
        }
    }
}

impl<A: CalibratedClock, B: CalibratedClock> ClockSynchronization<A, B> {
    pub fn to_a(&self, t: B::Instant) -> A::Instant {
        self.a.add_i64_ns(self.at, self.b.sub_i64_ns(t, self.bt))
    }

    pub fn to_b(&self, t: A::Instant) -> B::Instant {
        self.b.add_i64_ns(self.bt, self.a.sub_i64_ns(t, self.at))
    }

    pub fn epoch_a(&self) -> A::Instant {
        self.at
    }

    pub fn epoch_b(&self) -> B::Instant {
        self.bt
    }

    pub fn a(&self) -> A {
        self.a
    }

    pub fn b(&self) -> B {
        self.b
    }
}
