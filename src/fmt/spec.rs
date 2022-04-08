// Copyright (c) 2022 Tony Barbitta
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use once_cell::sync::OnceCell;
use regex::Regex;

fn arg_name_regex() -> &'static Regex {
    static REGEX: OnceCell<Regex> = OnceCell::new();
    REGEX.get_or_init(|| {
        Regex::new(r"[a-zA-Z]{1}[a-zA-Z0-9_]*").expect("Unable to compile arg name regex")
    })
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
    pub align: Alignment,
    pub width: Option<usize>,
}

mod detail {
    pub type LeftParse = (Option<String>, Option<usize>);
    pub type RightParse = (super::Alignment, Option<usize>);
    pub type FullParse = (LeftParse, RightParse);
}

impl FormatSpec {
    pub(crate) fn new(fmt_start: usize, spec_no: usize, spec_str: &str) -> crate::Result<Self> {
        if spec_str == "{}" {
            return Ok(Self {
                fmt_pos: fmt_start,
                spec_num: spec_no,
                arg_name: None,
                arg_num: None,
                align: Alignment::Left,
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
            return Ok(Self {
                fmt_pos: fmt_start,
                spec_num: spec_no,
                arg_name: None,
                arg_num: None,
                align: Alignment::Left,
                width: None,
            });
        }

        let ((name, num), (align, width)) = Self::parse_spec(spec_str, inner)?;
        Ok(Self {
            fmt_pos: fmt_start,
            spec_num: spec_no,
            arg_name: name,
            arg_num: num,
            align,
            width,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.arg_num.is_none()
            && self.arg_name.is_none()
            && self.align == Alignment::Left
            && self.width.is_none()
    }

    fn parse_spec(entire_spec: &str, inner: &str) -> crate::Result<detail::FullParse> {
        if let Some(colon_pos) = inner.find(':') {
            let (left, rest) = inner.split_at(colon_pos);
            let mut right = &rest[1..];
            let left_side = Self::parse_spec_left(entire_spec, left)?;
            let right_parsed = Self::parse_spec_right(entire_spec, right)?;
            Ok((left_side, right_parsed))
        } else {
            let parsed = Self::parse_spec_left(entire_spec, inner)?;
            Ok((parsed, (Alignment::Left, None)))
        }
    }

    fn parse_spec_left(entire: &str, input: &str) -> crate::Result<detail::LeftParse> {
        if input.is_empty() {
            Ok((None, None))
        } else if let Ok(num) = input.parse::<usize>() {
            Ok((None, Some(num)))
        } else if arg_name_regex().is_match(input) {
            Ok((Some(input.to_string()), None))
        } else {
            eprintln!("Unable to parse left side of colon in spec: {}", entire);
            Err(crate::Error::bad_spec(entire))
        }
    }

    fn parse_spec_right(entire: &str, input: &str) -> crate::Result<detail::RightParse> {
        let mut right = input;
        let align = if right.starts_with(['<', '>', '^']) {
            let a = match right.chars().next().unwrap() {
                '<' => Alignment::Left,
                '>' => Alignment::Right,
                '^' => Alignment::Center,
                _ => unreachable!(),
            };
            right = &right[1..];
            a
        } else {
            // TODO: Should this be None? Should align be Alignment instead of Option<Alignment>?
            Alignment::Left
        };

        let width = if right.is_empty() {
            None
        } else if let Ok(n) = right.parse::<usize>() {
            if n == 0 {
                eprintln!("Format spec is zero width: {}", entire);
                return Err(crate::Error::zero_width(entire));
            }
            Some(n)
        } else {
            eprintln!("Unable to parse right side of colon in spec: {}", entire);
            return Err(crate::Error::bad_spec(entire));
        };

        Ok((align, width))
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
        assert_eq!(spec.align, Alignment::Left);
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

        let spec = FormatSpec::new(0, 0, "{}}");
        assert!(spec.is_err());

        let spec = FormatSpec::new(0, 0, "{{}");
        assert!(spec.is_err());

        let spec = FormatSpec::new(0, 0, "{1:0}");
        assert!(spec.is_err());
    }

    #[test]
    fn basic_usages() {
        let spec = FormatSpec::new(0, 0, "{}").expect("error parsing {}");
        assert!(spec.is_empty());
        assert_eq!(spec.align, Alignment::Left);
        assert_eq!(spec.width, None);
        assert_eq!(spec.arg_num, None);
        assert_eq!(spec.arg_name, None);

        let spec = FormatSpec::new(0, 0, "{0}").expect("error parsing {0}");
        assert!(!spec.is_empty());
        assert_eq!(spec.align, Alignment::Left);
        assert_eq!(spec.width, None);
        assert_eq!(spec.arg_num, Some(0));
        assert_eq!(spec.arg_name, None);

        let spec = FormatSpec::new(0, 0, "{10}").expect("error parsing {10}");
        assert!(!spec.is_empty());
        assert_eq!(spec.align, Alignment::Left);
        assert_eq!(spec.width, None);
        assert_eq!(spec.arg_num, Some(10));
        assert_eq!(spec.arg_name, None);

        let spec = FormatSpec::new(0, 0, "{name}").expect("error parsing {name}");
        assert!(!spec.is_empty());
        assert_eq!(spec.align, Alignment::Left);
        assert_eq!(spec.width, None);
        assert_eq!(spec.arg_num, None);
        assert_eq!(spec.arg_name, Some("name".to_string()));

        let spec = FormatSpec::new(0, 0, "{:>}").expect("error parsing {:>}");
        assert!(!spec.is_empty());
        assert_eq!(spec.align, Alignment::Right);
        assert_eq!(spec.width, None);
        assert_eq!(spec.arg_num, None);
        assert_eq!(spec.arg_name, None);

        let spec = FormatSpec::new(0, 0, "{:1}").expect("error parsing {:1}");
        assert!(!spec.is_empty());
        assert_eq!(spec.align, Alignment::Left);
        assert_eq!(spec.width, Some(1));
        assert_eq!(spec.arg_num, None);
        assert_eq!(spec.arg_name, None);

        let spec = FormatSpec::new(0, 0, "{:10}").expect("error parsing {:10}");
        assert!(!spec.is_empty());
        assert_eq!(spec.align, Alignment::Left);
        assert_eq!(spec.width, Some(10));
        assert_eq!(spec.arg_num, None);
        assert_eq!(spec.arg_name, None);

        let spec = FormatSpec::new(0, 0, "{name:^}").expect("error parsing {name:^}");
        assert!(!spec.is_empty());
        assert_eq!(spec.align, Alignment::Center);
        assert_eq!(spec.width, None);
        assert_eq!(spec.arg_num, None);
        assert_eq!(spec.arg_name, Some("name".to_string()));

        let spec = FormatSpec::new(0, 0, "{2:>}").expect("error parsing {2:>}");
        assert!(!spec.is_empty());
        assert_eq!(spec.align, Alignment::Right);
        assert_eq!(spec.width, None);
        assert_eq!(spec.arg_num, Some(2));
        assert_eq!(spec.arg_name, None);

        let spec = FormatSpec::new(0, 0, "{10:<}").expect("error parsing {10:<}");
        assert!(!spec.is_empty());
        assert_eq!(spec.align, Alignment::Left);
        assert_eq!(spec.width, None);
        assert_eq!(spec.arg_num, Some(10));
        assert_eq!(spec.arg_name, None);

        let spec = FormatSpec::new(0, 0, "{name:^10}").expect("error parsing {name:^10}");
        assert!(!spec.is_empty());
        assert_eq!(spec.align, Alignment::Center);
        assert_eq!(spec.width, Some(10));
        assert_eq!(spec.arg_num, None);
        assert_eq!(spec.arg_name, Some("name".to_string()));

        let spec = FormatSpec::new(0, 0, "{2:>5}").expect("error parsing {2:>5}");
        assert!(!spec.is_empty());
        assert_eq!(spec.align, Alignment::Right);
        assert_eq!(spec.width, Some(5));
        assert_eq!(spec.arg_num, Some(2));
        assert_eq!(spec.arg_name, None);

        let spec = FormatSpec::new(0, 0, "{10:<1}").expect("error parsing {10:<1}");
        assert!(!spec.is_empty());
        assert_eq!(spec.align, Alignment::Left);
        assert_eq!(spec.width, Some(1));
        assert_eq!(spec.arg_num, Some(10));
        assert_eq!(spec.arg_name, None);

        let spec = FormatSpec::new(0, 0, "{name:>0}");
        assert!(spec.is_err());
    }
}
