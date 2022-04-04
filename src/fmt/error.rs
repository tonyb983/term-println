// Copyright (c) 2022 Tony Barbitta
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[derive(Debug)]
pub enum Error {
    InvalidFormat,
    InvalidSpec(String),
    InvalidArgNumber(String),
    IncorrectNumberOfArgs,
    Other(String),
}

impl Error {
    pub fn bad_arg_num(requested_num: usize, arg_count: usize) -> Self {
        Self::InvalidArgNumber(format!(
            "Arg number {} was requested, but only {} args were provided",
            requested_num, arg_count
        ))
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::InvalidFormat => write!(f, "Invalid format"),
            Error::IncorrectNumberOfArgs => write!(f, "Incorrect number of arguments"),
            Error::InvalidSpec(msg) => write!(f, "Invalid format specifier: {}", msg),
            Error::Other(s) => write!(f, "{}", s),
            Error::InvalidArgNumber(s) => write!(f, "Invalid argument number: {}", s),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
