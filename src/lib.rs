#![doc = include_str!("../README.md")]

use std::error::Error as StdError;
use std::fmt;

use derive_more::{Deref, DerefMut};

/// An explicit collection of Errors.
///
/// In order to be more concise, consider simply using [Errors].
pub type ErrorCollection = Errors;

/// A collection of multiple `anyhow::Error`s.
///
/// This is helpful because we often don't want to bail at the first error.
/// For example, take a simple method:
///
/// ```
/// # use anyhow::{bail, Result};
/// # #[derive(Debug, Clone, Copy)]
/// # struct Header { hash: u64 }
/// #
/// # fn read_header(raw: &[u8]) -> Result<Header> {
/// #     Ok(Header { hash: 0 })
/// # }
/// #
/// # fn hasher(contents: &str) -> u64 {
/// #     0
/// # }
/// #
/// fn check_file_integrity(raw: Vec<u8>) -> anyhow::Result<()> {
///   if raw.len() < 123 {
///      bail!("Data too short")
///   }
///
///   let Header { hash: expected_hash } = read_header(&raw[0..123])?;
///   let contents = str::from_utf8(&raw[123..])?;
///
///   if contents.len() < 2 {
///      bail!("Contents too short")
///   }
///
///   let data_hash = hasher(&contents);
///   if expected_hash != data_hash {
///      bail!("Header hash mismatch: {expected_hash} {data_hash}")
///   }
///
///   Ok(())
/// }
/// ```
///
/// We want the error to describe how the file is corrupted, but in our code we return
/// at the very first instance of an error. We are loosing valuable information during
/// runtime that could help debug a problem!
///
/// An [Errors] collection can help with this problem:
///
/// ```
/// # use anyhow::{anyhow, bail, Result};
/// # use error_collection::Errors;
/// # #[derive(Debug, Clone, Copy)]
/// # struct Header { hash: u64 }
/// #
/// # fn read_header(raw: &[u8]) -> Result<Header> {
/// #     Ok(Header { hash: 0 })
/// # }
/// #
/// # fn hasher(contents: &str) -> u64 {
/// #     0
/// # }
/// #
/// fn check_file_integrity(raw: Vec<u8>) -> anyhow::Result<()> {
///   if raw.len() < 123 {
///      bail!("Data too short")
///   }
///
///   let mut errors = Errors::new();
///
///   // Convert the results to options
///   let header = errors.collect(read_header(&raw[0..123]));
///   let contents = errors.collect(str::from_utf8(&raw[123..]));
///
///   if let Some(contents) = contents && contents.len() < 2 {
///     errors.append("Contents too short");
///   }
///
///   if let Some(data_hash) = contents.map(hasher) &&
///      let Some(Header { hash }) = header &&
///      hash != data_hash {
///      errors.push(anyhow!("Header hash mismatch: {hash} {data_hash}"));
///   }
///
///   errors.as_result() // Ok(()) if there are no errors
/// }
/// ```
///
/// ```text
/// 2 errors:
///    1. Missing lightbulb
///    2. Camera needs film
/// ```
#[derive(Debug, Default, Deref, DerefMut)]
pub struct Errors(pub Vec<anyhow::Error>);

impl Errors {
    /// Creates an empty error collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pushes an error to the back of this collection.
    ///
    /// Note: Pushing an Errors will nest that collection in this one.
    /// See [Self::append] for an alternative that avoids nesting.
    pub fn push(&mut self, err: impl Into<anyhow::Error>) {
        self.0.push(err.into());
    }

    /// Appends another collection of [Errors] to the back of this one.
    ///
    /// Tip: You can append from `Options` and `Results` that implement `Into<anyhow::Error>`.
    pub fn append(&mut self, err: impl Into<Self>) {
        self.0.append(&mut err.into().0);
    }

    /// Unwraps the error contained in this Result. Errors get "collected" into the collection, and out comes the optional result.
    pub fn collect<T, E>(&mut self, result: Result<T, E>) -> Option<T>
    where
        E: Into<anyhow::Error>,
    {
        match result {
            Ok(value) => Some(value),
            Err(err) => {
                // Flatten out Errors
                self.append(err.into());
                None
            }
        }
    }

    /// Consumes the collection and returns the inner vector.
    pub fn into_vec(self) -> Vec<anyhow::Error> {
        self.0
    }

    /// Consumes the collection and returns a result.
    pub fn as_result(mut self) -> anyhow::Result<()> {
        match self.len() {
            0 => Ok(()),
            1 => Err(self.pop().unwrap()),
            _ => Err(self.into()),
        }
    }
}

const PADDING: usize = 3;

impl fmt::Display for Errors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_collection(f, self, 0)
    }
}

/// Custom formatter for Errors
fn format_collection(f: &mut fmt::Formatter<'_>, errors: &Errors, indent: usize) -> fmt::Result {
    if errors.is_empty() {
        writeln!(f, "none")
    } else if errors.len() == 1 {
        format_error(f, &errors[0], indent)
    } else {
        writeln!(f, "{} errors:", errors.len())?;
        for (idx, error) in errors.iter().enumerate() {
            write!(f, "{}{}. ", spaces(indent + PADDING), idx + 1)?;
            match error.downcast_ref::<Errors>() {
                None => format_error(f, error, indent + PADDING)?,
                Some(errors) => format_collection(f, errors, indent + PADDING)?,
            }
        }
        Ok(())
    }
}

/// Custom formatter for an anyhow::Error nested in a collection
fn format_error(f: &mut fmt::Formatter<'_>, error: &anyhow::Error, indent: usize) -> fmt::Result {
    let padding = spaces(indent + PADDING);
    let error_string = if f.alternate() {
        format!("{:#}", error)
    } else {
        format!("{}", error)
    };
    for (idx, line) in error_string.split('\n').enumerate() {
        let padding = if idx == 0 { "" } else { padding };
        writeln!(f, "{padding}{line}")?;
    }
    Ok(())
}

/// Zero-alloc version of " ".repeat(x)
fn spaces(indent: usize) -> &'static str {
    &"                                "[..indent.min(32)]
}

impl StdError for Errors {}

impl From<&str> for Errors {
    fn from(value: &str) -> Self {
        Self(vec![anyhow::anyhow!("{value}")])
    }
}

impl From<String> for Errors {
    fn from(value: String) -> Self {
        Self(vec![anyhow::anyhow!(value)])
    }
}

impl<T> From<Option<T>> for Errors
where
    T: Into<anyhow::Error>,
{
    fn from(result: Option<T>) -> Self {
        match result {
            Some(err) => err.into().into(),
            None => Self::default(),
        }
    }
}

impl<T, E> From<Result<T, E>> for Errors
where
    E: Into<anyhow::Error>,
{
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(_) => Self::default(),
            Err(err) => err.into().into(),
        }
    }
}

impl From<Vec<anyhow::Error>> for Errors {
    fn from(errors: Vec<anyhow::Error>) -> Self {
        Self(errors)
    }
}

impl From<anyhow::Error> for Errors {
    fn from(error: anyhow::Error) -> Self {
        match error.downcast::<Self>() {
            Ok(errors) => errors,
            Err(error) => Self(vec![error]),
        }
    }
}

impl<T> From<Errors> for anyhow::Result<T>
where
    T: Default,
{
    fn from(mut value: Errors) -> Self {
        match value.len() {
            0 => Ok(T::default()),
            1 => Err(value.pop().unwrap()),
            _ => Err(value.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use anyhow::anyhow;

    use super::*;

    #[test]
    fn push() {
        let mut nested = Errors::new();
        nested.push(anyhow!("Generic error 1"));
        nested.push(anyhow!("Generic error 2"));
        nested.push(anyhow!("Generic error 3"));

        let mut errors = Errors::new();
        errors.push(nested);
        errors.push(anyhow!("Generic error 4"));
        errors.push(io::Error::from_raw_os_error(22));

        assert_eq!(errors.len(), 3);
    }

    #[test]
    fn append() {
        let mut nested = Errors::new();
        nested.append(vec![anyhow!("Generic error 1"), anyhow!("Generic error 2")]);

        let mut errors = Errors::new();
        errors.append(nested);
        errors.append(anyhow!("Generic error 3"));

        assert_eq!(errors.len(), 3);
    }

    #[test]
    fn collect() {
        let mut errors = Errors::new();

        let result: Result<(), anyhow::Error> = Ok(());
        assert_eq!(errors.collect(result), Some(()));

        let result: Result<(), anyhow::Error> = Err(anyhow!("Generic error 1"));
        assert_eq!(errors.collect(result), None);

        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn collect_nested() {
        let mut nested = Errors::new();
        nested.push(anyhow!("Generic error 1"));
        nested.push(anyhow!("Generic error 2"));
        nested.push(anyhow!("Generic error 3"));

        let mut errors = Errors::new();

        let result: Result<(), Errors> = Err(nested);
        assert_eq!(errors.collect(result), None);

        assert_eq!(errors.len(), 3);
    }

    #[test]
    fn fmt() {
        let mut child = Errors::new();
        child.push(anyhow!("Generic error 2"));
        child.push(anyhow!("Generic error 3\nnew line"));
        child.push(Errors(vec![anyhow!("Generic error 4")]));

        let mut parent = Errors::new();
        parent.push(child);
        parent.push(io::Error::from_raw_os_error(1));

        let mut errors = Errors::new();
        errors.push(anyhow!("Generic error 1\nnew line"));
        errors.push(parent);

        assert_eq!(
            format!("{errors:#}"),
            "2 errors:
                1. Generic error 1
                   new line
                2. 2 errors:
                   1. 3 errors:
                      1. Generic error 2
                      2. Generic error 3
                         new line
                      3. Generic error 4
                   2. Operation not permitted (os error 1)\n"
                .replace("\n             ", "\n")
        );
    }
}
