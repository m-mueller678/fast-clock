# fast-clock

Low-overhead timing for benchmarking without hiding the details from you.

Most timing crates wrap the clocks behind a convenient but opaque API.
`fast-clock` exposes the raw time source directly and offers some utilities to make them easier to work with.
There is no global state, no automatic fallbacks, no automatic calibration, and no automatic conversion to `std`'s `Duration`.
`fast-clock` supports `no-std`, `no-alloc`, and targets without floating-point support.

## Clocks

### TSC (x86_64)

The x86_64 timestamp counter can be read with a single instruction.
The frequency of the TSC varies by CPU, and for some CPUs even changes over time or between cores.
On Linux, `fast-clock` provides functionality to check whether the TSC frequency is constant.

```rust
// On Linux, verifies the kernel considers the TSC stable across cores.
let tsc = Tsc::try_new_linux_sys().unwrap();

// Calibrate against std::time::Instant (sleeps ~10 ms once at startup).
let (calibration, sync) = U64Calibration::new_with_std_instant(
    &tsc,
    std::time::Duration::from_millis(10),
);

let t0 = tsc.now();
// ... timed section ...
let t1 = tsc.now();

let ticks = WrappingU64Time::instant_sub(t1, t0);
let ns: u64 = calibration.convert_to_ns(ticks);
```

### Generic timer (aarch64)

The aarch64 generic timer can be read with a single instruction.
Its frequency is reported by a register, so no calibration is needed.

TODO example

### Converting between clock domains

The `sync` value returned by the calibration in the previous example is a `ClockSynchronization` that maps
instants between the TSC domain and `std::time::Instant`. This is useful when
correlating TSC measurements with wall-clock timestamps or other clocks.

```rust
// Convert a TSC instant to std::time::Instant.
let wall = sync.to_a(t0, &InherentlyCalibrated, &calibration);
```

`ClockSynchronization` is general: it works between any two clock domains.

### std::time::Instant / SystemTime

`std_clocks::InstantClock` and `std_clocks::SystemClock` wrap the standard
library clocks. Their durations are already in nanoseconds, so no calibration is
needed.

## Contributions
Contributions adding more clocks are welcome.
