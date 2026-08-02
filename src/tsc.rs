use crate::{
    Clock, DurationCallibration,
    primitive::{WrappingPrimitiveDuration, WrappingPrimitiveInstant, WrappingPrimitiveTime},
};
use std::time::Instant;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Tsc(());

impl Clock for Tsc {
    type Time = WrappingPrimitiveTime<i64>;
    type Callibration = CalibratedTsc;

    #[inline(always)]
    fn now(self) -> WrappingPrimitiveInstant<i64> {
        WrappingPrimitiveInstant(unsafe { core::arch::x86_64::_rdtsc() } as i64)
    }
}

#[derive(Copy, Clone)]
pub struct CalibratedTsc {
    ns_per_cycle: f64,
    tsc: Tsc,
}

impl Clock for CalibratedTsc {
    type Time = WrappingPrimitiveTime<i64>;
    type Callibration = Self;

    fn now(self) -> WrappingPrimitiveInstant<i64> {
        self.tsc.now()
    }
}

impl DurationCallibration<WrappingPrimitiveDuration<i64>> for CalibratedTsc {
    fn convert_to_i64_ns(&self, d: WrappingPrimitiveDuration<i64>) -> i64 {
        let cycles = d.0;
        debug_assert!(cycles >= 0);
        (cycles as f64 * self.ns_per_cycle).round() as i64
    }
    fn convert_from_i64_ns(&self, ns: i64) -> WrappingPrimitiveDuration<i64> {
        let cycles = (ns as f64 / self.ns_per_cycle).round() as i64;
        WrappingPrimitiveDuration(cycles)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct TscUnavailable;

impl core::fmt::Display for TscUnavailable {
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("No stable TSC available")
    }
}

impl Tsc {
    pub fn try_new_assume_stable() -> Result<Self, TscUnavailable> {
        let edx = core::arch::x86_64::__cpuid(1).edx;
        if (edx & (1 << 4)) != 0 {
            Ok(Tsc(()))
        } else {
            Err(TscUnavailable)
        }
    }

    #[cfg(target_os = "linux")]
    pub fn try_new_linux_sys() -> Result<Self, TscUnavailable> {
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
                debug_assert!(ns_per_cycle > 0.0);
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
