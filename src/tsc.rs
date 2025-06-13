use crate::{CalibratedClock, Clock};
use std::time::Instant;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct TscInstant(i64);

impl PartialOrd for TscInstant {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.0.cmp(&other.0))
    }
}

impl Ord for TscInstant {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0).wrapping_sub(other.0).cmp(&0)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Tsc(());

impl Clock for Tsc {
    type Instant = TscInstant;
    fn now(self) -> Self::Instant {
        TscInstant(unsafe { core::arch::x86_64::_rdtsc() } as i64)
    }
}

#[derive(Copy, Clone)]
pub struct CalibratedTsc {
    ns_per_cycle: f64,
    tsc: Tsc,
}

#[derive(thiserror::Error, Debug)]
#[error("No stable TSC available")]
#[non_exhaustive]
pub struct TscUnavailable;

impl Tsc {
    pub fn try_new() -> Result<Self, TscUnavailable> {
        let stable_tsc_detected = std::fs::read_to_string(
            "/sys/devices/system/clocksource/clocksource0/available_clocksource",
        )
        .is_ok_and(|x| x.contains("tsc"));
        if stable_tsc_detected {
            Ok(Tsc(()))
        } else {
            Err(TscUnavailable)
        }
    }

    pub fn calibrate(self) -> CalibratedTsc {
        let mut old_cycles = 0.0;
        loop {
            let t1 = Instant::now();
            let tsc1 = self.now();
            let mut t2;
            let mut tsc2;
            let cycles_per_ns = loop {
                t2 = Instant::now();
                tsc2 = self.now();
                let elapsed_nanos = (t2 - t1).as_nanos();
                let elapsed_cycles = tsc2.0.wrapping_sub(tsc1.0);
                if elapsed_nanos > 10_000_000 && elapsed_cycles > 0 {
                    break elapsed_cycles as f64 / elapsed_nanos as f64;
                }
            };
            let delta = f64::abs(cycles_per_ns - old_cycles);
            if delta / cycles_per_ns < 0.00001 {
                let ns_per_cycle = cycles_per_ns.recip();
                return CalibratedTsc {
                    ns_per_cycle,
                    tsc: self,
                };
            } else {
                old_cycles = cycles_per_ns;
            }
        }
    }
}

impl From<CalibratedTsc> for Tsc {
    fn from(value: CalibratedTsc) -> Self {
        value.tsc
    }
}

impl CalibratedClock for CalibratedTsc {
    fn sub_i64_ns(self, lhs: Self::Instant, rhs: Self::Instant) -> i64 {
        let d = lhs.0.wrapping_sub(rhs.0);
        (d as f64 * self.ns_per_cycle).round() as i64
    }

    fn add_i64_ns(self, base: Self::Instant, offset: i64) -> Self::Instant {
        TscInstant((offset as f64 / self.ns_per_cycle) as i64 + base.0)
    }
}

impl Clock for CalibratedTsc {
    type Instant = TscInstant;
    fn now(self) -> Self::Instant {
        self.tsc.now()
    }
}
