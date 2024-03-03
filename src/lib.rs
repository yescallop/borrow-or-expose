#![warn(missing_docs, rust_2018_idioms)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![no_std]

//! Traits for either borrowing from or exposing a reference from a value.
//!
//! # Walkthrough
//!
//! Suppose that you have a struct `Text<T>` where `T` may be `&str` or `String`.
//! You want to implement a generic method `as_str` on `Text` which returns
//! a longest-living reference to the inner string.
//! This is when [`BorrowOrExpose`] comes in handy:
//!
//! ```
//! use borrow_or_expose::BorrowOrExpose;
//!
//! struct Text<T>(T);
//!
//! impl<'i, 'o, T: BorrowOrExpose<'i, 'o, str>> Text<T> {
//!     fn as_str(&'i self) -> &'o str {
//!         self.0.borrow_or_expose()
//!     }
//! }
//!
//! // The returned reference, which is borrowed from `*t`, lives as long as `t`.
//! fn owned_as_str(t: &Text<String>) -> &str {
//!     t.as_str()
//! }
//!
//! // The returned reference, which is exposed from `t`, lives longer than `t`.
//! fn borrowed_as_str(t: Text<&str>) -> &str {
//!     t.as_str()
//! }
//! ```
//!
//! The [`BorrowOrExpose`] trait takes two lifetime parameters `'i`, `'o`,
//! and a type parameter `T`. Its [`borrow_or_expose`] method takes `&'i self`
//! and returns `&'o T`. You can use the trait to write your own "borrowing or
//! reference-exposing" functions, like the `as_str` method in the above example.
//!
//! The lifetime parameters on [`BorrowOrExpose`] can be quite restrictive when
//! reference-exposing behavior is not needed, such as in a [`fmt::Display`] implementation.
//! In such cases, [`Boe`] should be used instead:
//!
//! [`borrow_or_expose`]: BorrowOrExpose::borrow_or_expose
//! [`fmt::Display`]: core::fmt::Display
//!
//! ```
//! use borrow_or_expose::{Boe, BorrowOrExpose};
//! use std::fmt;
//!
//! struct Text<T>(T);
//!
//! impl<'i, 'o, T: BorrowOrExpose<'i, 'o, str>> Text<T> {
//!     fn as_str(&'i self) -> &'o str {
//!         self.0.borrow_or_expose()
//!     }
//! }
//!
//! impl<T: Boe<str>> fmt::Display for Text<T> {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         f.write_str(self.as_str())
//!     }
//! }
//! ```
//!
//! In the above example, the `as_str` method is also available on `Text<T>`
//! where `T: Boe<str>`, because [`BorrowOrExpose`] is implemented on
//! all types that implement [`Boe`]. It also works the other way round
//! because [`Boe`] is a supertrait of [`BorrowOrExpose`].
//!
//! Note that [`Boe<T>`] is also implemented on other types
//! such as `T`, `&mut T`, [`Box<T>`], [`Cow<'_, T>`], [`Rc<T>`], and [`Arc<T>`].
//! Consider adding extra trait bounds, preferably on a function that
//! constructs your type, if this is not desirable.
//!
//! [`Box<T>`]: alloc::boxed::Box
//! [`Cow<'_, T>`]: alloc::borrow::Cow
//! [`Rc<T>`]: alloc::rc::Rc
//! [`Arc<T>`]: alloc::sync::Arc
//!
//! You can also implement [`Boe`] on your own type, for example:
//!
//! ```
//! use borrow_or_expose::Boe;
//!
//! struct Text<'a>(&'a str);
//!
//! impl<'a> Boe<str> for Text<'a> {
//!     type Ref<'i> = &'a str where Self: 'i;
//!     
//!     fn borrow_or_expose_gat(&self) -> Self::Ref<'_> {
//!         self.0
//!     }
//! }
//! ```
//!
//! # Relation between [`Boe`], [`Borrow`] and [`AsRef`]
//!
//! [`Boe`] has the same signature as [`Borrow`] and [`AsRef`], but [`Boe`] is different in a few aspects:
//!
//! - The implementation of [`Boe`] for `&T` exposes the reference with lifetime unchanged
//!   instead of borrowing from it.
//! - [`Boe`] does not have extra requirements on [`Eq`], [`Ord`] and [`Hash`](core::hash::Hash)
//!   implementations as [`Borrow`] does. For this reason, you generally should not rely solely
//!   on [`Boe`] to implement [`Borrow`].
//! - Despite being safe to implement, [`Boe`] is not designed to be eagerly implemented as [`AsRef`] is.
//!   This crate only provides implementations of [`Boe`] on types that currently implement [`Borrow`]
//!   in the standard library. If this is too restrictive, feel free to copy the code pattern
//!   from this crate to your own codebase.
//!
//! [`Borrow`]: core::borrow::Borrow
//!
//! # Crate features
//!
//! - `std` (default): Enables [`Boe`] implementations
//!   on [`OsString`](std::ffi::OsString) and [`PathBuf`](std::path::PathBuf).

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

/// Types from whose values references may be borrowed or exposed.
///
/// See the [crate-level documentation](crate) for more details.
pub trait Boe<T: ?Sized> {
    /// The resulting reference type (must be `&T`), which may outlive `'i`.
    type Ref<'i>: Ref<T>
    where
        Self: 'i;

    /// Borrows from or exposes a reference from the value,
    /// returning a reference of type [`Self::Ref`].
    fn borrow_or_expose_gat(&self) -> Self::Ref<'_>;
}

/// A helper trait for writing "borrowing or reference-exposing" functions.
///
/// See the [crate-level documentation](crate) for more details.
pub trait BorrowOrExpose<'i, 'o, T: ?Sized>: Boe<T> {
    /// Borrows from or exposes a reference from the value.
    fn borrow_or_expose(&'i self) -> &'o T;
}

impl<'i, 'o, T: ?Sized, B> BorrowOrExpose<'i, 'o, T> for B
where
    B: Boe<T> + ?Sized + 'i,
    B::Ref<'i>: 'o,
{
    #[inline]
    fn borrow_or_expose(&'i self) -> &'o T {
        (self.borrow_or_expose_gat() as B::Ref<'i>).cast()
    }
}

impl<'a, T: ?Sized> Boe<T> for &'a T {
    type Ref<'i> = &'a T where Self: 'i;

    #[inline]
    fn borrow_or_expose_gat(&self) -> Self::Ref<'_> {
        self
    }
}

macro_rules! impl_boe {
    ($($(#[$attr:meta])? $({$($params:tt)*})? $ty:ty => $target:ty $(where {$($bounds:tt)*})?)*) => {
        $(
            $(#[$attr])?
            impl $(<$($params)*>)? Boe<$target> for $ty $(where $($bounds)*)? {
                type Ref<'i> = &'i $target where Self: 'i;

                #[inline]
                fn borrow_or_expose_gat(&self) -> Self::Ref<'_> {
                    self
                }
            }
        )*
    };
}

impl_boe! {
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
