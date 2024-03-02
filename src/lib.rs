#![warn(missing_docs, rust_2018_idioms)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![no_std]

//! Traits for types whose values when dereferenced may outlive themselves.
//!
//! # Walkthrough
//!
//! Suppose that you have a struct `Text<T>` where `T` may be `&str` or `String`.
//! You want to implement a generic method `as_str` on `Text` which returns
//! a longest-living reference to the inner string.
//! This is when [`OutlivingDeref`] comes in handy:
//!
//! ```
//! use outliving_deref::OutlivingDeref;
//!
//! struct Text<T>(T);
//!
//! impl<'i, 'o, T: OutlivingDeref<'i, 'o, str>> Text<T> {
//!     fn as_str(&'i self) -> &'o str {
//!         self.0.outliving_deref()
//!     }
//! }
//!
//! // The returned reference lives longer than `t`.
//! fn borrowed_as_str(t: Text<&str>) -> &str {
//!     t.as_str()
//! }
//!
//! // The returned reference lives as long as `t`.
//! fn owned_as_str(t: &Text<String>) -> &str {
//!     t.as_str()
//! }
//! ```
//!
//! The [`OutlivingDeref`] trait takes two lifetime parameters `'i`, `'o`,
//! and a type parameter `T`. Its [`outliving_deref`] method
//! takes `&'i self` and returns `&'o T`. You may use the trait to implement
//! your own "outliving-behaved" functions, like the `as_str` in the above example.
//!
//! The lifetime parameters on [`OutlivingDeref`] can be quite restrictive when
//! outliving behavior is not needed, such as in a [`fmt::Display`] implementation.
//! In such cases, [`Old`] should be used instead:
//!
//! [`outliving_deref`]: OutlivingDeref::outliving_deref
//! [`fmt::Display`]: core::fmt::Display
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
//! In the above example, the `as_str` method is also available on `Text<T>`
//! where `T: Old<str>`, because [`OutlivingDeref`] is implemented on
//! all types that implement [`Old`]. It also works the other way round
//! because [`Old`] is a supertrait of [`OutlivingDeref`].
//!
//! Note that [`Old<T>`] is also implemented on other types
//! such as `T`, `&mut T`, [`Box<T>`], [`Cow<'_, T>`], [`Rc<T>`], and [`Arc<T>`].
//! Consider adding extra trait bounds, preferably on a function that
//! constructs your type, if this is not desirable.
//!
//! [`Box<T>`]: alloc::boxed::Box
//! [`Cow<'_, T>`]: alloc::borrow::Cow
//! [`Rc<T>`]: alloc::rc::Rc
//! [`Arc<T>`]: alloc::sync::Arc
//!
//! You may also implement [`Old`] on your own type, for example:
//!
//! ```
//! use outliving_deref::Old;
//!
//! struct Text<'a>(&'a str);
//!
//! impl<'a> Old<str> for Text<'a> {
//!     type Ref<'i> = &'a str where Self: 'i;
//!     
//!     fn outliving_deref_assoc(&self) -> Self::Ref<'_> {
//!         self.0
//!     }
//! }
//! ```
//!
//! # Crate features
//!
//! - `std` (default): Enables [`std`] support.

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

trait Ref<T: ?Sized> {
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

/// Types whose values when dereferenced may outlive themselves.
///
/// See the [crate-level documentation](crate) for more details.
pub trait Old<T: ?Sized> {
    /// The resulting reference type (must be `&T`), which may outlive `'i`.
    #[allow(private_bounds)]
    type Ref<'i>: Ref<T>
    where
        Self: 'i;

    /// Dereferences the value, returning a reference of type [`Self::Ref`].
    fn outliving_deref_assoc(&self) -> Self::Ref<'_>;
}

/// [`Old`] with extra lifetime parameters.
///
/// See the [crate-level documentation](crate) for more details.
pub trait OutlivingDeref<'i, 'o, T: ?Sized>: Old<T> {
    /// Dereferences the value.
    fn outliving_deref(&'i self) -> &'o T;
}

impl<'i, 'o, T: ?Sized, O> OutlivingDeref<'i, 'o, T> for O
where
    O: Old<T> + ?Sized + 'i,
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
