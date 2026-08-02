use crate::{CallibratedClock, Clock, DurationCallibration, Time};

pub struct ClockSynchronization<A: Time, B: Time> {
    at: A::Instant,
    bt: B::Instant,
}

impl<A: Time, B: Time> ClockSynchronization<A, B> {
    pub fn new_aba<CA, CB>(a: &CallibratedClock<CA>, b: &CB) -> Self
    where
        CA: Clock<Time = A> + DurationCallibration<A::Duration> + Copy,
        A::Instant: Copy,
        A::Duration: Copy,
        CB: Clock<Time = B> + Copy,
        B::Instant: Copy,
    {
        let (a0, bt, da) = (0..3)
            .map(|_| {
                let a0 = a.clock.now();
                let bt = b.now();
                let a1 = a.clock.now();
                let d = A::instant_sub(a1, a0);
                let da = a.callibration.convert_to_i64_ns(d);
                (a0, bt, da)
            })
            .min_by_key(|(.., da)| *da)
            .unwrap();
        ClockSynchronization {
            bt,
            at: A::mixed_add(a0, a.callibration.convert_from_i64_ns(da / 2)),
        }
    }

    pub fn to_a<CA, CB>(&self, t: B::Instant, a: &CA, b: &CB) -> A::Instant
    where
        CA: DurationCallibration<A::Duration> + Copy,
        A::Instant: Copy,
        A::Duration: Copy,
        CB: DurationCallibration<B::Duration> + Copy,
        B::Instant: Copy,
        B::Duration: Copy,
    {
        let a = *a;
        let b = *b;
        let d_b = B::instant_sub(t, self.bt);
        let ns = b.convert_to_i64_ns(d_b);
        A::mixed_add(self.at, a.convert_from_i64_ns(ns))
    }

    pub fn to_b<CA, CB>(&self, t: A::Instant, a: &CA, b: &CB) -> B::Instant
    where
        CA: DurationCallibration<A::Duration> + Copy,
        A::Instant: Copy,
        A::Duration: Copy,
        CB: DurationCallibration<B::Duration> + Copy,
        B::Instant: Copy,
        B::Duration: Copy,
    {
        let a = *a;
        let b = *b;
        let d_a = A::instant_sub(t, self.at);
        let ns = a.convert_to_i64_ns(d_a);
        B::mixed_add(self.bt, b.convert_from_i64_ns(ns))
    }

    pub fn epoch_a(&self) -> A::Instant
    where
        A::Instant: Copy,
    {
        self.at
    }

    pub fn epoch_b(&self) -> B::Instant
    where
        B::Instant: Copy,
    {
        self.bt
    }
}

impl<A: Time, B: Time> Clone for ClockSynchronization<A, B>
where
    A::Instant: Clone,
    B::Instant: Clone,
{
    fn clone(&self) -> Self {
        ClockSynchronization {
            at: self.at.clone(),
            bt: self.bt.clone(),
        }
    }
}

impl<A: Time, B: Time> Copy for ClockSynchronization<A, B>
where
    A::Instant: Copy,
    B::Instant: Copy,
{
}
