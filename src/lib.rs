#![warn(missing_docs, rust_2018_idioms)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![no_std]

//! Traits for either borrowing or sharing data.
//!
//! # Walkthrough
//!
//! Suppose that you have a generic type which either owns some data or holds a reference to them.
//! You want to implement on this type a method taking `&self` which either borrows from `*self`
//! or from behind a reference it holds. A naive way to do this would be
//! to duplicate the method declaration:
//!
//! ```
//! struct Text<T>(T);
//!
//! impl Text<String> {
//!     // The returned reference is borrowed from `*self`.
//!     fn as_str(&self) -> &str {
//!         &self.0
//!     }
//! }
//!
//! impl<'a> Text<&'a str> {
//!     // The returned reference is borrowed from `*self.0`.
//!     fn as_str(&self) -> &'a str {
//!         self.0
//!     }
//! }
//! ```
//!
//! However, when you add more methods to `Text`, it would be
//! intolerably verbose to duplicate them for every `T`.
//! This crate thus provides a [`BorrowOrShare`] trait which can be used to
//! simplify the above code by making the `as_str` method generic over `T`:
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
//! // The returned reference is borrowed from `*t`.
//! fn owned_as_str(t: &Text<String>) -> &str {
//!     t.as_str()
//! }
//!
//! // The returned reference is borrowed from `*t.0`.
//! fn borrowed_as_str<'a>(t: &Text<&'a str>) -> &'a str {
//!     t.as_str()
//! }
//! ```
//!
//! The [`BorrowOrShare`] trait takes two lifetime parameters `'i`, `'o`,
//! and a type parameter `T`. When used, it implies an "associated lifetime bound":
//! for `Self = String` it implies `'i: 'o`, whereas for `Self = &'a str`
//! it implies `'a: 'o`.
//! The trait is also implemented for other types, which we'll cover later.
//!
//! On the trait is a [`borrow_or_share`] method which turns
//! `&'i self` into `&'o T`. You use it to write your own
//! "data borrowing-or-sharing" functions. A typical usage would be
//! to require a `BorrowOrShare<'i, 'o, str>` bound on a type parameter `T`
//! in an `impl` block of your type, within which you implement
//! a method that turns `&'i self` into some type with lifetime `'o`,
//! by calling the [`borrow_or_share`] method on some value of `T`
//! contained in `self` and further processing the returned `&'o str`.
//!
//! [`borrow_or_share`]: BorrowOrShare::borrow_or_share
//!
//! The lifetime parameters on [`BorrowOrShare`] can be quite restrictive
//! when data-sharing behavior is not needed, such as in an [`AsRef`]
//! implementation. In such cases, [`Bos`] should be used instead:
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
//! Note that [`Bos<T>`] is also [implemented for other types][impls]
//! such as `&mut T`, [`Box<T>`], and [`Rc<T>`].
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
//! # Limitations
//!
//! This crate only provides implementations of [`Bos`] for types that
//! currently implement [`Borrow`] in the standard library, except for
//! the blanket implementation. If this is too restrictive, feel free
//! to copy the code pattern from this crate as you wish.
//!
//! [`Borrow`]: core::borrow::Borrow
//!
//! # Crate features
//!
//! - `std` (disabled by default): Enables [`Bos`] implementations for
//!   [`OsString`](std::ffi::OsString) and [`PathBuf`](std::path::PathBuf).

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

/// A trait for either borrowing or sharing data.
///
/// See the [crate-level documentation](crate) for more details.
pub trait Bos<T: ?Sized> {
    /// The resulting reference type. May only be `&T`.
    type Ref<'this>: Ref<T>
    where
        Self: 'this;

    /// Borrows from `*this` or from behind a reference it holds,
    /// returning a reference of type [`Self::Ref`].
    fn borrow_or_share(this: &Self) -> Self::Ref<'_>;
}

/// A helper trait for writing "data borrowing-or-sharing" functions.
///
/// See the [crate-level documentation](crate) for more details.
pub trait BorrowOrShare<'i, 'o, T: ?Sized>: Bos<T> {
    /// Borrows from `*self` or from behind a reference it holds.
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
    // A blanket impl would show up everywhere in the
    // documentation of a dependent crate, which is noisy.
    // So we're omitting it for the moment.
    // {T: ?Sized} T => T

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
