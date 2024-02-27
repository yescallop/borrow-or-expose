#![warn(missing_docs, rust_2018_idioms)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![no_std]

//! Traits for types whose values when dereferenced may outlive themselves.
//!
//! # Examples
//!
//! Consider the following code:
//!
//! ```
//! use std::fmt;
//!
//! struct Text<T>(T);
//!
//! impl<'a> Text<&'a str> {
//!     fn as_str(&self) -> &'a str {
//!         self.0
//!     }
//! }
//!
//! impl Text<String> {
//!     fn as_str(&self) -> &str {
//!         &self.0
//!     }
//! }
//!
//! impl fmt::Display for Text<&str> {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         f.write_str(self.as_str())
//!     }
//! }
//!
//! impl fmt::Display for Text<String> {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         f.write_str(self.as_str())
//!     }
//! }
//! ```
//!
//! Using this crate, we may generalize the above code to:
//!
//! ```
//! use outliving_deref::{Old, OutlivingDeref};
//! use std::fmt;
//!
//! struct Text<T>(T);
//!
//! impl<'i, 'o, T: OutlivingDeref<'i, 'o, str>> Text<T> {
//!     fn as_str(&'i self) -> &'o str {
//!         self.0.outliving_deref()
//!     }
//! }
//!
//! impl<T: Old<str>> fmt::Display for Text<T> {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         f.write_str(self.as_str())
//!     }
//! }
//! ```
//!
//! Note that [`Old<T>`] is also implemented on other types
//! such as `T`, `&mut T`, [`Box<T>`], [`Cow<'_, T>`], [`Rc<T>`], and [`Arc<T>`].
//! Consider adding extra trait bounds if this is not desirable.
//!
//! [`Box<T>`]: alloc::boxed::Box
//! [`Cow<'_, T>`]: alloc::borrow::Cow
//! [`Rc<T>`]: alloc::rc::Rc
//! [`Arc<T>`]: alloc::sync::Arc
//!
//! # Crate features
//!
//! - `std` (default): Enables [`std`] support.

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

mod internal {
    pub trait Ref<T: ?Sized> {
        fn cast<'a>(self) -> &'a T
        where
            Self: 'a;
    }

    impl<T: ?Sized> Ref<T> for &T {
        #[inline]
        fn cast<'a>(self) -> &'a T
        where
            Self: 'a,
        {
            self
        }
    }
}

use internal::Ref;

/// Types whose values when dereferenced may outlive themselves.
///
/// See the [crate-level documentation](crate) for more details.
pub trait Old<T: ?Sized> {
    /// The resulting reference type (must be `&T`), which may outlive `'i`.
    type Ref<'i>: Ref<T>
    where
        Self: 'i;

    /// Dereferences the value, returning a reference of type [`Self::Ref`].
    fn outliving_deref_assoc(&self) -> Self::Ref<'_>;
}

/// [`Old`] with lifetime parameters.
///
/// See the [crate-level documentation](crate) for more details.
pub trait OutlivingDeref<'i, 'o, T: ?Sized>: Old<T> {
    /// Dereferences the value.
    fn outliving_deref(&'i self) -> &'o T;
}

impl<'i, 'o, T: ?Sized, O: Old<T> + 'i> OutlivingDeref<'i, 'o, T> for O
where
    O::Ref<'i>: 'o,
{
    #[inline]
    fn outliving_deref(&'i self) -> &'o T {
        (self.outliving_deref_assoc() as O::Ref<'i>).cast()
    }
}

impl<'a, T: ?Sized> Old<T> for &'a T {
    type Ref<'i> = &'a T where Self: 'i;

    #[inline]
    fn outliving_deref_assoc(&self) -> Self::Ref<'_> {
        self
    }
}

macro_rules! impl_old {
    ($($(#[$attr:meta])? $({$($params:tt)*})? $ty:ty => $target:ty $(where {$($bounds:tt)*})?)*) => {
        $(
            $(#[$attr])?
            impl $(<$($params)*>)? Old<$target> for $ty $(where $($bounds)*)? {
                type Ref<'i> = &'i $target where Self: 'i;

                #[inline]
                fn outliving_deref_assoc(&self) -> Self::Ref<'_> {
                    self
                }
            }
        )*
    };
}

impl_old! {
    {T: ?Sized} T => T
    {T: ?Sized} &mut T => T

    {T, const N: usize} [T; N] => [T]
    {T} alloc::vec::Vec<T> => [T]

    alloc::string::String => str
    alloc::ffi::CString => core::ffi::CStr

    #[cfg(feature = "std")]
    std::ffi::OsString => std::ffi::OsStr
    #[cfg(feature = "std")]
    std::path::PathBuf => std::path::Path

    {T: ?Sized} alloc::boxed::Box<T> => T
    {B: ?Sized + alloc::borrow::ToOwned} alloc::borrow::Cow<'_, B> => B
        where {B::Owned: core::borrow::Borrow<B>}

    {T: ?Sized} alloc::rc::Rc<T> => T
    {T: ?Sized} alloc::sync::Arc<T> => T
}
