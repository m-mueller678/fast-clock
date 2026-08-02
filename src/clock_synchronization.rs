use crate::{CalibratedClock, Clock, DurationCalibration, Time};

/// A pair of instants from two clocks that correspond to roughly the same point in time.
pub struct ClockSynchronization<A: Time, B: Time> {
    epoch_a: A::Instant,
    epoch_b: B::Instant,
}

impl<A: Time, B: Time> ClockSynchronization<A, B> {
    /// Creates a synchronization from a known pair of corresponding instants.
    pub fn new(epoch_a: A::Instant, epoch_b: B::Instant) -> Self {
        ClockSynchronization { epoch_a, epoch_b }
    }

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
            epoch_b: bt,
            epoch_a: A::mixed_add(a0, a.calibration.convert_from_ns(da / 2)),
        }
    }

    /// Converts an instant from clock B's domain to clock A's domain.
    ///
    /// Works for instants both before and after the synchronization epoch.
    pub fn to_a<CA, CB>(&self, t: B::Instant, a: &CA, b: &CB) -> A::Instant
    where
        CA: DurationCalibration<A::Duration>,
        CB: DurationCalibration<B::Duration>,
    {
        if B::instant_cmp(t, self.epoch_b).is_lt() {
            let d_b = B::instant_sub(self.epoch_b, t);
            A::mixed_sub(self.epoch_a, a.convert_from_ns(b.convert_to_ns(d_b)))
        } else {
            let d_b = B::instant_sub(t, self.epoch_b);
            A::mixed_add(self.epoch_a, a.convert_from_ns(b.convert_to_ns(d_b)))
        }
    }

    /// Converts an instant from clock A's domain to clock B's domain.
    ///
    /// Works for instants both before and after the synchronization epoch.
    pub fn to_b<CA, CB>(&self, t: A::Instant, a: &CA, b: &CB) -> B::Instant
    where
        CA: DurationCalibration<A::Duration>,
        CB: DurationCalibration<B::Duration>,
    {
        if A::instant_cmp(t, self.epoch_a).is_lt() {
            let d_a = A::instant_sub(self.epoch_a, t);
            B::mixed_sub(self.epoch_b, b.convert_from_ns(a.convert_to_ns(d_a)))
        } else {
            let d_a = A::instant_sub(t, self.epoch_a);
            B::mixed_add(self.epoch_b, b.convert_from_ns(a.convert_to_ns(d_a)))
        }
    }

    /// Converts an instant from clock B's domain to clock A's domain, assuming `t` is at
    /// or after the synchronization epoch.
    ///
    /// Possibly faster than [`to_a`](Self::to_a).
    /// Behavior when `t` is before the epoch is unspecified: The function may
    /// panic or return a meaningless value. Use [`to_a`](Self::to_a) if unsure.
    pub fn to_a_after_epoch<CA, CB>(&self, t: B::Instant, a: &CA, b: &CB) -> A::Instant
    where
        CA: DurationCalibration<A::Duration>,
        CB: DurationCalibration<B::Duration>,
    {
        let d_b = B::instant_sub(t, self.epoch_b);
        A::mixed_add(self.epoch_a, a.convert_from_ns(b.convert_to_ns(d_b)))
    }

    /// Converts an instant from clock A's domain to clock B's domain, assuming `t` is at
    /// or after the synchronization epoch.
    ///
    /// Possibly faster than [`to_b`](Self::to_b).
    /// Behavior when `t` is before the epoch is unspecified: The function may
    /// panic or return a meaningless value. Use [`to_b`](Self::to_b) if unsure.
    pub fn to_b_after_epoch<CA, CB>(&self, t: A::Instant, a: &CA, b: &CB) -> B::Instant
    where
        CA: DurationCalibration<A::Duration>,
        CB: DurationCalibration<B::Duration>,
    {
        let d_a = A::instant_sub(t, self.epoch_a);
        B::mixed_add(self.epoch_b, b.convert_from_ns(a.convert_to_ns(d_a)))
    }

    /// Returns the synchronization epoch in clock A's domain.
    pub fn epoch_a(&self) -> A::Instant {
        self.epoch_a
    }

    /// Returns the synchronization epoch in clock B's domain.
    pub fn epoch_b(&self) -> B::Instant {
        self.epoch_b
    }
}

impl<A: Time, B: Time> Clone for ClockSynchronization<A, B> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: Time, B: Time> Copy for ClockSynchronization<A, B> {}

impl<A: Time, B: Time> core::fmt::Debug for ClockSynchronization<A, B>
where
    A::Instant: core::fmt::Debug,
    B::Instant: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ClockSynchronization")
            .field("epoch_a", &self.epoch_a)
            .field("epoch_b", &self.epoch_b)
            .finish()
    }
}
