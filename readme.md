# fast-clock

This crate aims to provide access to various time sources for benchmarking purposes.
It tries to hide as little as possible from you:
There is no global state and no automatic fallback to alternative time sources.

Currently, only `std::time::{SytemTime, Instant}` and TSC are supported.