//! A shim crate to easily test code with `loom`.
//!
//! Import common types and modules like `UnsafeCell`, `thread`, `Arc`,
//! `AtomicI32`, etc from this crate, and run loom tests from the command line
//! with `cargo test --features loomy/enable`.
//!
//! ## Example
//!
//! The following module can be tested in two ways:
//!
//! ```sh
//! $ cargo test
//! $ cargo test --features loomy/enable
//! ```
//!
//! When `loomy/enable` is set, then the code will be tested as a loomy model,
//! otherwise all types default to their `std` equivalents, and the code will be
//! tested as normal.
//!
//! ```rust
//! // Note the use of `loomy` instead of `std` or `loom`.
//! use loomy::{
//!     hint,
//!     cell::UnsafeCell,
//!     sync::atomic::{AtomicBool, Ordering},
//! };
//!
//! pub struct SpinLock<T> {
//!     flag: AtomicBool,
//!     data: UnsafeCell<T>,
//! }
//!
//! unsafe impl<T> Send for SpinLock<T> {}
//! unsafe impl<T> Sync for SpinLock<T> {}
//!
//! impl<T> SpinLock<T> {
//!     pub fn new(t: T) -> Self {
//!         Self {
//!             flag: AtomicBool::new(false),
//!             data: UnsafeCell::new(t),
//!         }
//!     }
//!
//!     pub fn with<R, F: FnOnce(&mut T) -> R>(&self, f: F) -> R {
//!         while let Err(_) = self
//!             .flag
//!             .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
//!         {
//!             hint::spin_loop()
//!         }
//!
//!         let out = self.data.with_mut(move |t| unsafe { f(&mut *t) });
//!         self.flag.store(false, Ordering::Release);
//!         out
//!     }
//! }
//!
//! #[cfg(test)]
//! mod tests {
//! # }
//!     // Also using `loomy` instead of `loom` or `std`.
//!     use loomy::{thread, sync::Arc};
//!     # mod tmp {
//!     use super::*;
//!     # }
//!
//!     #[test]
//!     # fn mock() {}
//!     fn test_simple() {
//!         loomy::model(|| {
//!             let lock = Arc::new(SpinLock::new(123));
//!             let lock2 = Arc::clone(&lock);
//!
//!             let t = thread::spawn(move || {
//!                 lock2.with(|n| *n += 1);
//!             });
//!
//!             lock.with(|n| *n = 456);
//!
//!             let out = lock.with(|n| *n);
//!
//!             t.join().unwrap();
//!
//!             assert!(out == 456 || out == 457);
//!         });
//!     }
//!     # mod dummy {
//! }
//! # test_simple();
//! ```
//!
//! ## A note on `UnsafeCell`
//!
//! `UnsafeCell` in `loom` has a closure-based API. When using `std` types,
//! `UnsafeCell` is wrapped in order to provide the same API.

#[cfg(feature = "enable")]
mod imp {
    pub use loom::*;
}

#[cfg(not(feature = "enable"))]
mod imp {
    pub use std::{alloc, hint, sync, thread};

    pub mod cell {
        pub use std::cell::*;

        #[derive(Debug, Default)]
        pub struct UnsafeCell<T>(std::cell::UnsafeCell<T>);

        impl<T> From<T> for UnsafeCell<T> {
            #[inline(always)]
            fn from(t: T) -> Self {
                Self(std::cell::UnsafeCell::new(t))
            }
        }

        impl<T> UnsafeCell<T> {
            #[inline(always)]
            pub fn new(data: T) -> UnsafeCell<T> {
                UnsafeCell(std::cell::UnsafeCell::new(data))
            }

            #[inline(always)]
            pub fn with<R>(&self, f: impl FnOnce(*const T) -> R) -> R {
                f(self.0.get())
            }

            #[inline(always)]
            pub fn with_mut<R>(&self, f: impl FnOnce(*mut T) -> R) -> R {
                f(self.0.get())
            }
        }
    }

    #[inline(always)]
    pub fn model<F: FnOnce()>(f: F) {
        f()
    }
}

pub use self::imp::*;
