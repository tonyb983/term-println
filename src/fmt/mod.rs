// Copyright (c) 2022 Tony Barbitta
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

mod arg;
mod error;
mod formatter;
mod spec;

pub use arg::{FormatArg, FormatArgs};
pub use error::{Error, Result};
pub use formatter::Formatter;
pub use spec::{Alignment, FormatSpec};

use once_cell::sync::OnceCell;
use regex::Regex;

pub fn spec_regex() -> &'static Regex {
    static REGEX: OnceCell<Regex> = OnceCell::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r"(?x)
        \{
            (?P<Inner>
                (?P<ArgId>\d+|[a-zA-Z_]\w*)?
                (?P<FmtSpec>:
                    (?:
                        (?P<Fill>[^{}])?
                        (?P<Align>[<>\^])
                    )?
                    (?P<Sign>[+-s])?
                    (?P<AltForm>\#)?
                    (?P<NegPadding>[0])?
                    (?P<Width>
                        (?P<WidthNumber>\d+)|
                        (?P<WidthArgId>\{
                            (?P<WidthInner>
                                (?P<WidthArgInteger>\d+)|
                                (?P<WidthArgIdentifier>[a-zA-Z_]\w*)
                            )?
                        \})
                    )? # Width Group
                    (?:\.
                        (?P<Precision>
                            (?P<PrecisionNumber>\d+)|
                            (?P<PrecisionArgId>\{
                                (?P<PrecisionInner>
                                    (?P<PrecisionArgInteger>\d+)|
                                    (?P<PrecisionArgIdentifier>[a-zA-Z_]\w*)
                                )?
                            \})
                        )
                    )? # Precision Group
                    (?P<Type>[bBdoxXaAceEfFgGLps])?
                )*? # FmtSpec
            )*? # Inner
        \}",
        )
        .expect("Failed to compile spec regex")
    })
}

pub fn spec_regex_simple() -> &'static Regex {
    static REGEX: OnceCell<Regex> = OnceCell::new();
    REGEX.get_or_init(|| {
        Regex::new(r"\{(?P<Inner>(?P<ArgId>\d+|[a-zA-Z_]\w*)?)\}").expect("Failed to compile regex")
    })
}

pub fn spec_regex_brackets_only() -> &'static Regex {
    static REGEX: OnceCell<Regex> = OnceCell::new();
    // Match anything between brackets but as few as possible. Previously this was:
    //      Regex::new(r"\{.*\}")
    // but would hit the first bracket and match until the last bracket, ignoring any opening
    // and closings in between
    REGEX.get_or_init(|| Regex::new(r"\{.{0,}?\}").expect("Failed to compile regex"))
}
