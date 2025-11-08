/*
 * Copyright 2019 The Starlark in Rust Authors.
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::ops::Sub;
use std::time::Duration;

use allocative::Allocative;

/// Real `Instant` for production code, thread-local counter for tests and WASM.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Allocative)]
pub(crate) struct ProfilerInstant(
    #[cfg(all(not(test), not(target_arch = "wasm32")))] std::time::Instant,
    #[cfg(any(test, target_arch = "wasm32"))] u64, // Millis.
);

impl ProfilerInstant {
    #[cfg(test)]
    pub(crate) const TEST_TICK_MILLIS: u64 = 7;

    #[inline]
    pub(crate) fn now() -> Self {
        #[cfg(all(not(test), not(target_arch = "wasm32")))]
        {
            ProfilerInstant(std::time::Instant::now())
        }
        #[cfg(test)]
        {
            thread_local! {
                static NOW_MILLIS: std::cell::Cell<u64> = const { std::cell::Cell::new(100003) };
            }
            ProfilerInstant(NOW_MILLIS.with(|v| {
                let r = v.get();
                v.set(r + ProfilerInstant::TEST_TICK_MILLIS);
                r
            }))
        }
        #[cfg(all(target_arch = "wasm32", not(test)))]
        {
            // WASM doesn't have a monotonic clock, so we return a constant.
            // This means timing measurements will always be zero on WASM.
            ProfilerInstant(0)
        }
    }

    #[inline]
    pub(crate) fn duration_since(&self, earlier: ProfilerInstant) -> Duration {
        #[cfg(all(not(test), not(target_arch = "wasm32")))]
        {
            self.0.duration_since(earlier.0)
        }
        #[cfg(any(test, target_arch = "wasm32"))]
        {
            Duration::from_millis(self.0.checked_sub(earlier.0).unwrap())
        }
    }

    #[inline]
    pub(crate) fn elapsed(&self) -> Duration {
        #[cfg(all(not(test), not(target_arch = "wasm32")))]
        {
            self.0.elapsed()
        }
        #[cfg(any(test, target_arch = "wasm32"))]
        {
            ProfilerInstant::now().duration_since(*self)
        }
    }
}

impl Sub for ProfilerInstant {
    type Output = Duration;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.duration_since(rhs)
    }
}
