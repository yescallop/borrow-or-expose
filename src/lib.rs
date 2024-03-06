#![warn(missing_docs, rust_2018_idioms)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![no_std]

//! Traits for either borrowing data or sharing references.
//!
//! # Walkthrough
//!
//! Suppose that you have a struct `Text<T>` where `T` may be `String` or `&str`.
//! You want to implement a generic method `as_str` on `Text` which returns
//! a longest-living reference to the inner string.
//! This is when [`BorrowOrShare`] comes in handy:
//!
//! ```
//! use borrow_or_share::BorrowOrShare;
//!
//! struct Text<T>(T);
//!
//! impl<'i, 'o, T: BorrowOrShare<'i, 'o, str>> Text<T> {
//!     fn as_str(&'i self) -> &'o str {
//!         self.0.borrow_or_share()
//!     }
//! }
//!
//! // The returned reference is borrowed from `*t` and lives as long as `t`.
//! fn owned_as_str(t: &Text<String>) -> &str {
//!     t.as_str()
//! }
//!
//! // The returned reference is copied from `t.0` and lives longer than `t`.
//! fn borrowed_as_str(t: Text<&str>) -> &str {
//!     t.as_str()
//! }
//! ```
//!
//! The [`BorrowOrShare`] trait takes two lifetime parameters `'i`, `'o`,
//! and a type parameter `T`. Its [`borrow_or_share`] method takes `&'i self`
//! and returns `&'o T`. You can use the trait to write your own "data-borrowing or
//! reference-sharing" functions, like the `as_str` method in the above example.
//!
//! The lifetime parameters on [`BorrowOrShare`] can be quite restrictive when
//! reference-sharing behavior is not needed, such as in an [`AsRef`] implementation.
//! In such cases, [`Bos`] should be used instead:
//!
//! [`borrow_or_share`]: BorrowOrShare::borrow_or_share
//!
//! ```
//! use borrow_or_share::{BorrowOrShare, Bos};
//!
//! struct Text<T>(T);
//!
//! impl<'i, 'o, T: BorrowOrShare<'i, 'o, str>> Text<T> {
//!     fn as_str(&'i self) -> &'o str {
//!         self.0.borrow_or_share()
//!     }
//! }
//!
//! impl<T: Bos<str>> AsRef<str> for Text<T> {
//!     fn as_ref(&self) -> &str {
//!         self.as_str()
//!     }
//! }
//! ```
//!
//! In the above example, the `as_str` method is also available on `Text<T>`
//! where `T: Bos<str>`, because [`BorrowOrShare`] is implemented for
//! all types that implement [`Bos`]. It also works the other way round
//! because [`Bos`] is a supertrait of [`BorrowOrShare`].
//!
//! Note that [`Bos<T>`] is also [implemented for other types][impls] such as `T`,
//! `&mut T`, [`Box<T>`], and [`Rc<T>`].
//! Consider adding extra trait bounds, preferably on a function that
//! constructs your type, if this is not desirable.
//!
//! [impls]: Bos#foreign-impls
//! [`Box<T>`]: alloc::boxed::Box
//! [`Rc<T>`]: alloc::rc::Rc
//!
//! You can also implement [`Bos`] for your own type, for example:
//!
//! ```
//! use borrow_or_share::Bos;
//!
//! struct Text<'a>(&'a str);
//!
//! impl<'a> Bos<str> for Text<'a> {
//!     type Ref<'this> = &'a str where Self: 'this;
//!     
//!     fn borrow_or_share(this: &Self) -> Self::Ref<'_> {
//!         this.0
//!     }
//! }
//! ```
//!
//! # Relation between [`Bos`], [`Borrow`], and [`AsRef`]
//!
//! [`Bos`] is similar to [`Borrow`] and [`AsRef`], but different in a few aspects:
//!
//! - The implementation of [`Bos`] for `&T` copies the reference with lifetime unchanged
//!   instead of borrowing from it.
//! - [`Bos`] does not have extra requirements on [`Eq`], [`Ord`], and [`Hash`](core::hash::Hash)
//!   implementations as [`Borrow`] does.
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

/// A trait for either borrowing data or sharing references.
///
/// See the [crate-level documentation](crate) for more details.
pub trait Bos<T: ?Sized> {
    /// The resulting reference type. May only be `&'a T` where `'a: 'this`.
    type Ref<'this>: Ref<T>
    where
        Self: 'this;

    /// Borrows from a value or gets a shared reference from it,
    /// returning a reference of type [`Self::Ref`].
    fn borrow_or_share(this: &Self) -> Self::Ref<'_>;
}

/// A helper trait for writing "data-borrowing or reference-sharing" functions.
///
/// See the [crate-level documentation](crate) for more details.
pub trait BorrowOrShare<'i, 'o, T: ?Sized>: Bos<T> {
    /// Borrows from a value or gets a shared reference from it.
    fn borrow_or_share(&'i self) -> &'o T;
}

impl<'i, 'o, T: ?Sized, B> BorrowOrShare<'i, 'o, T> for B
where
    B: Bos<T> + ?Sized + 'i,
    B::Ref<'i>: 'o,
{
    #[inline]
    fn borrow_or_share(&'i self) -> &'o T {
        (B::borrow_or_share(self) as B::Ref<'i>).cast()
    }
}

impl<'a, T: ?Sized> Bos<T> for &'a T {
    type Ref<'this> = &'a T where Self: 'this;

    #[inline]
    fn borrow_or_share(this: &Self) -> Self::Ref<'_> {
        this
    }
}

macro_rules! impl_bos {
    ($($(#[$attr:meta])? $({$($params:tt)*})? $ty:ty => $target:ty)*) => {
        $(
            $(#[$attr])?
            impl $(<$($params)*>)? Bos<$target> for $ty {
                type Ref<'this> = &'this $target where Self: 'this;

                #[inline]
                fn borrow_or_share(this: &Self) -> Self::Ref<'_> {
                    this
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

    {T: ?Sized} alloc::rc::Rc<T> => T
    {T: ?Sized} alloc::sync::Arc<T> => T
}
