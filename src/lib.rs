#![warn(missing_docs, rust_2018_idioms)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![no_std]

//! Traits for types whose values when dereferenced may outlive themselves.
//!
//! # Crate features
//!
//! - `std` (default): Enables [`std`] support.
//!
//! # Examples
//!
//! Consider the following code:
//!
//! ```
//! use std::fmt;
//!
//! struct Uri<T>(T);
//!
//! impl<'a> Uri<&'a str> {
//!     fn as_str(&self) -> &'a str {
//!         self.0
//!     }
//! }
//!
//! impl Uri<String> {
//!     fn as_str(&self) -> &str {
//!         &self.0
//!     }
//! }
//!
//! impl fmt::Display for Uri<&str> {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         f.write_str(self.as_str())
//!     }
//! }
//!
//! impl fmt::Display for Uri<String> {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         f.write_str(self.as_str())
//!     }
//! }
//! ```
//!
//! Using this crate, we may generalize the above code to:
//!
//! ```
//! use outliving_deref::{OutDeref, OutDerefExt};
//! use std::fmt;
//!
//! struct Uri<T>(T);
//!
//! impl<'i, 'o, T: OutDerefExt<'i, 'o, str>> Uri<T> {
//!     fn as_str(&'i self) -> &'o str {
//!         self.0.outliving_deref()
//!     }
//! }
//!
//! impl<T: OutDeref<str>> fmt::Display for Uri<T> {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         f.write_str(self.as_str())
//!     }
//! }
//! ```

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
pub trait OutlivingDeref {
    /// The resulting type after dereferencing.
    type Target: ?Sized;

    /// The resulting reference type (must be `&Self::Target`), which may outlive `'i`.
    type Ref<'i>: Ref<Self::Target>
    where
        Self: 'i;

    /// Dereferences the value, returning a reference of type [`Self::Ref`].
    fn outliving_deref_assoc(&self) -> Self::Ref<'_>;
}

/// Short for <code>[OutlivingDeref]<Target = T></code>.
///
/// This trait is automatically implemented for all types that implement [`OutlivingDeref`].
pub trait OutDeref<T: ?Sized>: OutlivingDeref<Target = T> {}

impl<T: ?Sized, O: OutlivingDeref<Target = T>> OutDeref<T> for O {}

/// [`OutDeref`] with lifetime parameters.
///
/// This trait is automatically implemented for all types that implement [`OutlivingDeref`].
pub trait OutDerefExt<'i, 'o, T: ?Sized>: OutlivingDeref<Target = T> {
    /// Dereferences the value.
    fn outliving_deref(&'i self) -> &'o T;
}

impl<'i, 'o, T: ?Sized, O: OutlivingDeref<Target = T> + 'i> OutDerefExt<'i, 'o, T> for O
where
    O::Ref<'i>: 'o,
{
    #[inline]
    fn outliving_deref(&'i self) -> &'o T {
        (self.outliving_deref_assoc() as O::Ref<'i>).cast()
    }
}

impl<'a, T: ?Sized> OutlivingDeref for &'a T {
    type Target = T;
    type Ref<'i> = &'a T where Self: 'i;

    #[inline]
    fn outliving_deref_assoc(&self) -> Self::Ref<'_> {
        self
    }
}

macro_rules! impl_outliving_deref {
    ($($(#[$attr:meta])? $({$($params:tt)*})? $ty:ty => $target:ty $(where {$($bounds:tt)*})?)*) => {
        $(
            $(#[$attr])?
            impl $(<$($params)*>)? OutlivingDeref for $ty $(where $($bounds)*)? {
                type Target = $target;
                type Ref<'i> = &'i $target where Self: 'i;

                #[inline]
                fn outliving_deref_assoc(&self) -> Self::Ref<'_> {
                    self
                }
            }
        )*
    };
}

impl_outliving_deref! {
    alloc::ffi::CString => core::ffi::CStr
    #[cfg(feature = "std")]
    std::ffi::OsString => std::ffi::OsStr
    #[cfg(feature = "std")]
    std::path::PathBuf => std::path::Path
    alloc::string::String => str
    {B: ?Sized + alloc::borrow::ToOwned} alloc::borrow::Cow<'_, B> => B
        where {B::Owned: core::borrow::Borrow<B>}
    {T: ?Sized} alloc::boxed::Box<T> => T
    {T: ?Sized} alloc::rc::Rc<T> => T
    {T: ?Sized} alloc::sync::Arc<T> => T
    {T} alloc::vec::Vec<T> => [T]
}
