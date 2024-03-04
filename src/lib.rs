#![warn(missing_docs, rust_2018_idioms)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![no_std]

//! Traits for either borrowing from or copying a reference from within a value.
//!
//! # Walkthrough
//!
//! Suppose that you have a struct `Text<T>` where `T` may be `String` or `&str`.
//! You want to implement a generic method `as_str` on `Text` which returns
//! a longest-living reference to the inner string.
//! This is when [`BorrowOrSteal`] comes in handy:
//!
//! ```
//! use borrow_or_steal::BorrowOrSteal;
//!
//! struct Text<T>(T);
//!
//! impl<'i, 'o, T: BorrowOrSteal<'i, 'o, str>> Text<T> {
//!     fn as_str(&'i self) -> &'o str {
//!         self.0.borrow_or_steal()
//!     }
//! }
//!
//! // The returned reference, which is borrowed from `*t`, lives as long as `t`.
//! fn owned_as_str(t: &Text<String>) -> &str {
//!     t.as_str()
//! }
//!
//! // The returned reference, which is copied from within `t`, lives longer than `t`.
//! fn borrowed_as_str(t: Text<&str>) -> &str {
//!     t.as_str()
//! }
//! ```
//!
//! The [`BorrowOrSteal`] trait takes two lifetime parameters `'i`, `'o`,
//! and a type parameter `T`. Its [`borrow_or_steal`] method takes `&'i self`
//! and returns `&'o T`. You can use the trait to write your own "borrowing or
//! reference-copying" functions, like the `as_str` method in the above example.
//!
//! The lifetime parameters on [`BorrowOrSteal`] can be quite restrictive when
//! reference-copying behavior is not needed, such as in a [`fmt::Display`] implementation.
//! In such cases, [`Bos`] should be used instead:
//!
//! [`borrow_or_steal`]: BorrowOrSteal::borrow_or_steal
//! [`fmt::Display`]: core::fmt::Display
//!
//! ```
//! use borrow_or_steal::{Bos, BorrowOrSteal};
//! use std::fmt;
//!
//! struct Text<T>(T);
//!
//! impl<'i, 'o, T: BorrowOrSteal<'i, 'o, str>> Text<T> {
//!     fn as_str(&'i self) -> &'o str {
//!         self.0.borrow_or_steal()
//!     }
//! }
//!
//! impl<T: Bos<str>> fmt::Display for Text<T> {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         f.write_str(self.as_str())
//!     }
//! }
//! ```
//!
//! In the above example, the `as_str` method is also available on `Text<T>`
//! where `T: Bos<str>`, because [`BorrowOrSteal`] is implemented for
//! all types that implement [`Bos`]. It also works the other way round
//! because [`Bos`] is a supertrait of [`BorrowOrSteal`].
//!
//! Note that [`Bos<T>`] is also implemented for other types
//! such as `T`, `&mut T`, [`Box<T>`], [`Cow<'_, T>`], [`Rc<T>`], and [`Arc<T>`].
//! Consider adding extra trait bounds, preferably on a function that
//! constructs your type, if this is not desirable.
//!
//! [`Box<T>`]: alloc::boxed::Box
//! [`Cow<'_, T>`]: alloc::borrow::Cow
//! [`Rc<T>`]: alloc::rc::Rc
//! [`Arc<T>`]: alloc::sync::Arc
//!
//! You can also implement [`Bos`] for your own type, for example:
//!
//! ```
//! use borrow_or_steal::Bos;
//!
//! struct Text<'a>(&'a str);
//!
//! impl<'a> Bos<str> for Text<'a> {
//!     type Ref<'i> = &'a str where Self: 'i;
//!     
//!     fn borrow_or_steal_gat(&self) -> Self::Ref<'_> {
//!         self.0
//!     }
//! }
//! ```
//!
//! # Relation between [`Bos`], [`Borrow`] and [`AsRef`]
//!
//! [`Bos`] has the same signature as [`Borrow`] and [`AsRef`], but [`Bos`] is different in a few aspects:
//!
//! - The implementation of [`Bos`] for `&T` copies the reference with lifetime unchanged
//!   instead of borrowing from it.
//! - [`Bos`] does not have extra requirements on [`Eq`], [`Ord`] and [`Hash`](core::hash::Hash)
//!   implementations as [`Borrow`] does. For this reason, you generally should not rely solely
//!   on [`Bos`] to implement [`Borrow`].
//! - Despite being safe to implement, [`Bos`] is not meant to be eagerly implemented as [`AsRef`] is.
//!   This crate only provides implementations of [`Bos`] for types that currently implement [`Borrow`]
//!   in the standard library. If this is too restrictive, feel free to copy the code pattern
//!   from this crate as you wish.
//!
//! [`Borrow`]: core::borrow::Borrow
//!
//! # Crate features
//!
//! - `std` (default): Enables [`Bos`] implementations
//!   for [`OsString`](std::ffi::OsString) and [`PathBuf`](std::path::PathBuf).

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

/// A trait for borrowing data or copying references.
///
/// See the [crate-level documentation](crate) for more details.
pub trait Bos<T: ?Sized> {
    /// The resulting reference type (must be `&T`), which may outlive `'i`.
    type Ref<'i>: Ref<T>
    where
        Self: 'i;

    /// Borrows from or copies a reference from within the value,
    /// returning a reference of type [`Self::Ref`].
    fn borrow_or_steal_gat(&self) -> Self::Ref<'_>;
}

/// A helper trait for writing "borrowing or reference-copying" functions.
///
/// See the [crate-level documentation](crate) for more details.
pub trait BorrowOrSteal<'i, 'o, T: ?Sized>: Bos<T> {
    /// Borrows from or copies a reference from within the value.
    fn borrow_or_steal(&'i self) -> &'o T;
}

impl<'i, 'o, T: ?Sized, B> BorrowOrSteal<'i, 'o, T> for B
where
    B: Bos<T> + ?Sized + 'i,
    B::Ref<'i>: 'o,
{
    #[inline]
    fn borrow_or_steal(&'i self) -> &'o T {
        (self.borrow_or_steal_gat() as B::Ref<'i>).cast()
    }
}

impl<'a, T: ?Sized> Bos<T> for &'a T {
    type Ref<'i> = &'a T where Self: 'i;

    #[inline]
    fn borrow_or_steal_gat(&self) -> Self::Ref<'_> {
        self
    }
}

macro_rules! impl_bos {
    ($($(#[$attr:meta])? $({$($params:tt)*})? $ty:ty => $target:ty $(where {$($bounds:tt)*})?)*) => {
        $(
            $(#[$attr])?
            impl $(<$($params)*>)? Bos<$target> for $ty $(where $($bounds)*)? {
                type Ref<'i> = &'i $target where Self: 'i;

                #[inline]
                fn borrow_or_steal_gat(&self) -> Self::Ref<'_> {
                    self
                }
            }
        )*
    };
}

impl_bos! {
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
