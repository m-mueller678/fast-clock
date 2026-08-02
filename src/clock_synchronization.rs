use crate::{CalibratedClock, Clock, DurationCalibration, Time};

/// A pair of instants from two clocks that correspond to roughly the same point in time.
pub struct ClockSynchronization<A: Time, B: Time> {
    at: A::Instant,
    bt: B::Instant,
}

impl<A: Time, B: Time> ClockSynchronization<A, B> {
    /// Creates a synchronization by sampling both clocks in an A-B-A pattern.
    ///
    /// Multiple measurements are performed and the one with least error is selected.
    pub fn new_aba_calibrated<CA, CB>(a: &CalibratedClock<CA>, b: &CB) -> Self
    where
        CA: Clock<Time = A>,
        CB: Clock<Time = B>,
    {
        let (a0, bt, da) = (0..3)
            .map(|_| {
                loop {
                    let a0 = a.clock.now();
                    let bt = b.now();
                    let a1 = a.clock.now();
                    if A::instant_cmp(a0, a1).is_le() {
                        let d = A::instant_sub(a1, a0);
                        let da = a.calibration.convert_to_ns(d);
                        break (a0, bt, da);
                    }
                }
            })
            .min_by_key(|(.., da)| *da)
            .unwrap();
        ClockSynchronization {
            bt,
            at: A::mixed_add(a0, a.calibration.convert_from_ns(da / 2)),
        }
    }

    /// Converts an instant from clock B's domain to clock A's domain.
    pub fn to_a<CA, CB>(&self, t: B::Instant, a: &CA, b: &CB) -> A::Instant
    where
        CA: DurationCalibration<A::Duration>,
        CB: DurationCalibration<B::Duration>,
    {
        let d_b = B::instant_sub(t, self.bt);
        let ns = b.convert_to_ns(d_b);
        A::mixed_add(self.at, a.convert_from_ns(ns))
    }

    /// Converts an instant from clock A's domain to clock B's domain.
    pub fn to_b<CA, CB>(&self, t: A::Instant, a: &CA, b: &CB) -> B::Instant
    where
        CA: DurationCalibration<A::Duration>,
        CB: DurationCalibration<B::Duration>,
    {
        let d_a = A::instant_sub(t, self.at);
        let ns = a.convert_to_ns(d_a);
        B::mixed_add(self.bt, b.convert_from_ns(ns))
    }

    /// Returns the synchronization epoch in clock A's domain.
    pub fn epoch_a(&self) -> A::Instant {
        self.at
    }

    /// Returns the synchronization epoch in clock B's domain.
    pub fn epoch_b(&self) -> B::Instant {
        self.bt
    }
}

impl<A: Time, B: Time> Clone for ClockSynchronization<A, B> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: Time, B: Time> Copy for ClockSynchronization<A, B> {}
