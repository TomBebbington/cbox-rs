//! Provides a single type, `CBox`
extern crate libc;
use libc::{malloc, free, c_char, c_void, size_t};
use std::ffi::CStr;
use std::{fmt, ptr, str};
use std::ops::{Deref, DerefMut, Drop};
use std::marker::PhantomData;
/// Implemented by any type represented by a pointer that can be disposed
pub trait DisposeRef {
    /// What type this reference is to
    type RefTo;
    /// Destroy the contents at the pointer's location
    unsafe fn dispose(ptr: *mut Self::RefTo);
}

/// A wrapper for pointers made by C that have aliases in Rust
///
/// This is necessary to allow owned and borrowed representations of C types
/// to be represented by the same type as they are in C with little overhead
pub struct CBox<'a, D:?Sized> where D:DisposeRef+'a {
    ptr: *mut D::RefTo,
    marker: PhantomData<&'a ()>
}
impl<'a, D:?Sized> CBox<'a, D> where D:DisposeRef+'a {
    #[inline(always)]
    /// Wrap the pointer in a `CBox`
    pub fn new(ptr: *mut D::RefTo) -> Self {
        CBox {
            ptr: ptr,
            marker: PhantomData
        }
    }
    #[inline(always)]
    /// Returns the internal pointer
    pub fn unwrap(self) -> *mut D::RefTo {
        self.ptr
    }
}
impl<'a, D:?Sized> Drop for CBox<'a, D> where D:DisposeRef+'a {
    #[inline(always)]
    /// Run the destructor
    fn drop(&mut self) {
        unsafe { <D as DisposeRef>::dispose(self.ptr) }
    }
}
impl<'a, D> Deref for CBox<'a, D> where D:DisposeRef+'a, *mut D::RefTo:Into<&'a D> {
    type Target = D;
    fn deref(&self) -> &D {
        self.ptr.into()
    }
}
impl<'a, D> DerefMut for CBox<'a, D> where D:DisposeRef+'a, *mut D::RefTo:Into<&'a D>, *mut D::RefTo:Into<&'a mut D> {
    fn deref_mut(&mut self) -> &mut D {
        self.ptr.into()
    }
}
impl<'a> Deref for CBox<'a, str> {
    type Target = str;
    fn deref(&self) -> &str {
        unsafe {
            let text = CStr::from_ptr(self.ptr);
            str::from_utf8_unchecked(text.to_bytes())
        }
    }
}
impl<'a, 'b> From<&'a str> for CBox<'b, str> {
    fn from(text: &'a str) -> CBox<'b, str> {
        unsafe {
            let ptr = libc::malloc(text.len() as size_t + 1) as *mut c_char;
            ptr::copy(text.as_ptr() as *mut c_char, ptr, text.len());
            ptr::write(text.as_ptr().offset(text.len() as isize) as *mut c_char, 0);
            CBox::new(ptr)
        }
    }
}
impl<'a> fmt::Display for CBox<'a, str> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.deref())
    }
}
impl<'a> fmt::Debug for CBox<'a, str> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.deref())
    }
}
impl<'a, T> fmt::Display for CBox<'a, T> where T:fmt::Display+DisposeRef+'a, *mut T::RefTo:Into<&'a T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self as &T, fmt)
    }
}
impl<'a, T> fmt::Debug for CBox<'a, T> where T:fmt::Debug+DisposeRef+'a, *mut T::RefTo:Into<&'a T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self as &T, fmt)
    }
}
impl DisposeRef for str {
    type RefTo = c_char;
    unsafe fn dispose(ptr: *mut c_char) {
        free(ptr as *mut c_void)
    }
}
