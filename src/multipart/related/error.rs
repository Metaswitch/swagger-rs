//! Copyright 2016 mime-multipart Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.
//
// File defining Error enum used in the module.

use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::io;
use std::string::FromUtf8Error;

use httparse;
use hyper_0_10;

/// An error type for the `mime-multipart` crate.
pub enum Error {
    /// The Hyper request did not have a Content-Type header.
    NoRequestContentType,
    /// The Hyper request Content-Type top-level Mime was not `Multipart`.
    NotMultipart,
    /// The Content-Type header failed to specify boundary token.
    BoundaryNotSpecified,
    /// A multipart section contained only partial headers.
    PartialHeaders,
    /// The request headers ended pre-maturely.
    EofInMainHeaders,
    /// The request body ended prior to reaching the expected starting boundary.
    EofBeforeFirstBoundary,
    /// Missing CRLF after boundary.
    NoCrLfAfterBoundary,
    /// The request body ended prematurely while parsing headers of a multipart part.
    EofInPartHeaders,
    /// The request body ended prematurely while streaming a file part.
    EofInFile,
    /// The request body ended prematurely while reading a multipart part.
    EofInPart,
    /// An HTTP parsing error from a multipart section.
    Httparse(httparse::Error),
    /// An I/O error.
    Io(io::Error),
    /// An error was returned from Hyper.
    Hyper(hyper_0_10::Error),
    /// An error occurred during UTF-8 processing.
    Utf8(FromUtf8Error),
    /// An error occurred during character decoding
    Decoding(Cow<'static, str>),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<httparse::Error> for Error {
    fn from(err: httparse::Error) -> Error {
        Error::Httparse(err)
    }
}

impl From<hyper_0_10::Error> for Error {
    fn from(err: hyper_0_10::Error) -> Error {
        Error::Hyper(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Error {
        Error::Utf8(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Httparse(ref e) => format!("Httparse: {:?}", e).fmt(f),
            Error::Io(ref e) => format!("Io: {}", e).fmt(f),
            Error::Hyper(ref e) => format!("Hyper: {}", e).fmt(f),
            Error::Utf8(ref e) => format!("Utf8: {}", e).fmt(f),
            Error::Decoding(ref e) => format!("Decoding: {}", e).fmt(f),
            _ => format!("{}", self).fmt(f),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)?;
        if self.source().is_some() {
            write!(f, ": {:?}", self.source().unwrap())?; // recurse
        }
        Ok(())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::NoRequestContentType => "The Hyper request did not have a Content-Type header.",
            Error::NotMultipart => {
                "The Hyper request Content-Type top-level Mime was not multipart."
            }
            Error::BoundaryNotSpecified => {
                "The Content-Type header failed to specify a boundary token."
            }
            Error::PartialHeaders => "A multipart section contained only partial headers.",
            Error::EofInMainHeaders => "The request headers ended pre-maturely.",
            Error::EofBeforeFirstBoundary => {
                "The request body ended prior to reaching the expected starting boundary."
            }
            Error::NoCrLfAfterBoundary => "Missing CRLF after boundary.",
            Error::EofInPartHeaders => {
                "The request body ended prematurely while parsing headers of a multipart part."
            }
            Error::EofInFile => "The request body ended prematurely while streaming a file part.",
            Error::EofInPart => {
                "The request body ended prematurely while reading a multipart part."
            }
            Error::Httparse(_) => {
                "A parse error occurred while parsing the headers of a multipart section."
            }
            Error::Io(_) => "An I/O error occurred.",
            Error::Hyper(_) => "A Hyper error occurred.",
            Error::Utf8(_) => "A UTF-8 error occurred.",
            Error::Decoding(_) => "A decoding error occurred.",
        }
    }
}
