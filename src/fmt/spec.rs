// Copyright (c) 2022 Tony Barbitta
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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
    REGEX.get_or_init(|| Regex::new(r"\{.*\}").expect("Failed to compile regex"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub struct FormatSpec {
    pub fmt_pos: usize,
    pub spec_num: usize,
    pub arg_num: Option<usize>,
    pub arg_name: Option<String>,
    pub align: Option<Alignment>,
    pub width: Option<usize>,
}

impl FormatSpec {
    pub(crate) fn new(fmt_start: usize, spec_no: usize, spec_str: &str) -> crate::Result<Self> {
        if spec_str == "{}" {
            return Ok(Self {
                fmt_pos: fmt_start,
                spec_num: spec_no,
                arg_name: None,
                arg_num: None,
                align: None,
                width: None,
            });
        }

        if spec_str.contains("{{") || spec_str.contains("}}") {
            return Err(crate::Error::bad_spec(spec_str));
        }

        if !spec_str.starts_with('{') || !spec_str.ends_with('}') {
            return Err(crate::Error::bad_spec(spec_str));
        }

        let inner = spec_str.trim_start_matches('{').trim_end_matches('}');
        if inner.is_empty() {
            Ok(Self {
                fmt_pos: fmt_start,
                spec_num: spec_no,
                arg_name: None,
                arg_num: None,
                align: None,
                width: None,
            })
        } else if let Ok(num) = inner.parse::<usize>() {
            Ok(Self {
                fmt_pos: fmt_start,
                spec_num: spec_no,
                arg_name: None,
                arg_num: Some(num),
                align: None,
                width: None,
            })
        } else if let Some(colon) = inner.find(':') {
            let (left, rest) = inner.split_at(colon);
            let mut right = &rest[1..];
            let (arg_name, arg_num) = if left.is_empty() {
                (None, None)
            } else if let Ok(num) = left.parse::<usize>() {
                (None, Some(num))
            } else {
                (Some(left.to_string()), None)
                // return Err(crate::Error::InvalidSpec(spec_str.to_string()));
            };

            let align = if right.starts_with(['<', '>', '^']) {
                let a = match right.chars().next().unwrap() {
                    '<' => Alignment::Left,
                    '>' => Alignment::Right,
                    '^' => Alignment::Center,
                    _ => unreachable!(),
                };
                right = &right[1..];
                Some(a)
            } else {
                Some(Alignment::Left)
            };

            let width = if right.is_empty() {
                None
            } else if let Ok(n) = right.parse::<usize>() {
                Some(n)
            } else {
                return Err(crate::Error::bad_spec(spec_str));
            };

            Ok(Self {
                fmt_pos: fmt_start,
                spec_num: spec_no,
                arg_name,
                arg_num,
                align,
                width,
            })
        } else {
            Err(crate::Error::bad_spec(spec_str))
        }
    }

    pub fn is_empty(&self) -> bool {
        self.arg_num.is_none()
            && (self.align.is_none() || matches!(self.align, Some(Alignment::Left)))
            && self.width.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

    #[test]
    fn empty_brackets() {
        let spec = FormatSpec::new(0, 0, "{}").expect("Unable to create format spec from {}");
        assert_eq!(spec.arg_num, None);
        assert_eq!(spec.align, None);
        assert_eq!(spec.width, None);
        assert!(spec.is_empty());
    }

    #[test]
    fn bad_specs() {
        let spec = FormatSpec::new(0, 0, "{{");
        assert!(spec.is_err());

        let spec = FormatSpec::new(0, 0, "}");
        assert!(spec.is_err());

        let spec = FormatSpec::new(0, 0, "}{");
        assert!(spec.is_err());
    }
}
