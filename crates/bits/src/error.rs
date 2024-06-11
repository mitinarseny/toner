use core::fmt::{Debug, Display};
use std::error::Error as StdError;

use thiserror::Error as ThisError;

/// **De**/**ser**ialization error
pub trait Error: StdError + Sized {
    /// Returns a custom error from given message
    fn custom<T>(msg: T) -> Self
    where
        T: Display;

    /// Wraps current error in given context
    fn context<C>(self, context: C) -> Self
    where
        C: Display;
}

/// Adapter for providing context on [`Result`]
pub trait ResultExt: Sized {
    /// Wrap [`Err`] in context by calling given function
    fn with_context<C>(self, context: impl FnOnce() -> C) -> Self
    where
        C: Display;

    /// Wrap [`Err`] in given context
    #[inline]
    fn context<C>(self, context: C) -> Self
    where
        C: Display,
    {
        self.with_context(move || context)
    }
}

impl<T, E> ResultExt for Result<T, E>
where
    E: Error,
{
    #[inline]
    fn with_context<C>(self, context: impl FnOnce() -> C) -> Result<T, E>
    where
        C: Display,
    {
        self.map_err(move |err| err.context(context()))
    }
}

/// [`String`]-backed [`Error`]
#[derive(Debug, ThisError)]
#[error("{0}")]
pub struct StringError(String);

impl Error for StringError {
    #[inline]
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self(msg.to_string())
    }

    #[inline]
    fn context<C>(self, context: C) -> Self
    where
        C: Display,
    {
        Self(format!("{context}: {self}"))
    }
}
