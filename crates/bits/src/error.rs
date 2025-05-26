use core::fmt::{Debug, Display};
use std::{error::Error as StdError, io};

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
pub trait Context: Sized {
    type Ok;
    type Error: Error;

    /// Wrap [`Err`] in context by calling given function
    fn with_context<C>(self, context: impl FnOnce() -> C) -> Result<Self::Ok, Self::Error>
    where
        C: Display;

    /// Wrap [`Err`] in given context
    #[inline]
    fn context<C>(self, context: C) -> Result<Self::Ok, Self::Error>
    where
        C: Display,
    {
        self.with_context(move || context)
    }
}

impl<T, E> Context for Result<T, E>
where
    E: Error,
{
    type Ok = T;
    type Error = E;

    #[inline]
    fn with_context<C>(self, context: impl FnOnce() -> C) -> Result<T, E>
    where
        C: Display,
    {
        self.map_err(move |err| err.context(context()))
    }
}

impl<T> Context for Option<T> {
    type Ok = T;

    type Error = StringError;

    #[inline]
    fn with_context<C>(self, context: impl FnOnce() -> C) -> Result<Self::Ok, Self::Error>
    where
        C: Display,
    {
        self.ok_or_else(context).map_err(Error::custom)
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

impl AsRef<str> for StringError {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl Error for io::Error {
    #[inline]
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::other(msg.to_string())
    }

    #[inline]
    fn context<C>(self, context: C) -> Self
    where
        C: Display,
    {
        Self::new(self.kind(), format!("{context}: {self}"))
    }
}
