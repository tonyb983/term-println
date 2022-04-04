// Copyright (c) 2022 Tony Barbitta
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

mod arg;
mod error;
mod spec;

pub use arg::FormatArg;
pub use error::{Error, Result};
pub use spec::FormatSpec;

#[derive(Debug, Clone)]
pub struct Formatter {
    expected_args: u8,
    fmt_str: String,
    fmt_spec: Vec<spec::FormatSpec>,
}

impl Formatter {
    pub fn format(fmt_str: &str, args: &[&str]) -> crate::Result<String> {
        let formatter = Formatter::new(fmt_str)?;
        formatter.generate(args)
    }

    pub fn format_owned(fmt_str: &str, args: &[String]) -> crate::Result<String> {
        let formatter = Formatter::new(fmt_str)?;
        let ref_args = args.iter().map(|s| s.as_str()).collect::<Vec<_>>();
        formatter.generate(ref_args.as_slice())
    }

    pub fn new(fmt_str: &str) -> error::Result<Self> {
        let (s, spec) = match Self::parse_fmt(fmt_str) {
            Ok((s, spec)) => (s, spec),
            Err(err) => return Err(err),
        };

        // TODO: account for number of positional args and multiple specs referring to the same position
        let expected = spec.len() as u8;
        Ok(Self {
            expected_args: expected,
            fmt_str: s,
            fmt_spec: spec,
        })
    }

    pub fn expected_args(&self) -> u8 {
        self.expected_args
    }

    pub fn generate<S: std::fmt::Display>(&self, args: &[S]) -> crate::Result<String> {
        let args = args.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let mut inserted = 0usize;
        let mut offset = 0usize;
        let mut mods = Vec::new();

        for spec in &self.fmt_spec {
            let insert = if let Some(n) = spec.arg_num {
                match args.get(n) {
                    Some(s) => s,
                    None => return Err(crate::Error::bad_arg_num(n, args.len())),
                }
            } else {
                let s = match args.get(inserted) {
                    Some(s) => s,
                    None => return Err(crate::Error::bad_arg_num(inserted, args.len())),
                };
                inserted += 1;
                s
            };

            // TODO: Use the alignment and width here to modify `insert`

            mods.push((insert.clone(), spec.fmt_pos));
        }

        let mut output = self.fmt_str.clone();
        for (insert, pos) in mods.iter().rev() {
            if !output.is_char_boundary(*pos) {
                panic!("position {} is not a char boundary for output string {} (attempting to insert {})", pos, output, insert);
            }

            output.insert_str(*pos, insert);
        }

        Ok(output)
    }

    fn parse_fmt(s: &str) -> crate::Result<(String, Vec<FormatSpec>)> {
        // Other options for placeholders are:
        // ' ' - Negative Acknowledgement (Dec 21, Oct 025, Hex 15)
        // ' ' - Synchronous Idle (Dec 22, Oct 026, Hex 16)
        // ' ' - End of Medium (Dec 25, Oct 031, Hex 19)
        // ' ' - File Separator (Dec 28, Oct 034, Hex 1C)
        // ' ' - Group Separator (Dec 29, Oct 035, Hex 1D)
        // ' ' - Record Separator (Dec 30, Oct 036, Hex 1E)
        // ' ' - Unit Separator (Dec 31, Oct 037, Hex 1F)
        const LEFT_PLACEHOLDER: &str = "\u{1}";
        const RIGHT_PLACEHOLDER: &str = "\u{2}";

        if s.contains(LEFT_PLACEHOLDER) || s.contains(RIGHT_PLACEHOLDER) {
            let l_pos = s.find(LEFT_PLACEHOLDER);
            let r_pos = s.find(RIGHT_PLACEHOLDER);
            let l_msg = if let Some(pos) = l_pos {
                format!("It DOES contain the LEFT placeholder at position {}", pos)
            } else {
                "It DOES NOT contain the LEFT placeholder".to_string()
            };
            let r_msg = if let Some(pos) = r_pos {
                format!("It DOES contain the RIGHT placeholder at position {}", pos)
            } else {
                "It DOES NOT contain the RIGHT placeholder".to_string()
            };
            panic!("\nInput string contains one of the left or right placeholders! \n\tInput string is '{}'. \n\t{}. \n\t{}.", s, l_msg, r_msg);
        }

        let mut locs = spec::spec_regex_simple().capture_locations();
        let mut pos = 0usize;
        let mut spec_num = 0usize;
        let mut specs = Vec::new();
        let mut spec_ranges = Vec::new();
        let mut removed = 0usize;

        // TODO: This might be hella stupid or maybe even dangerous, do more research!
        // Here I am substituting in random unicode characters as placeholders for the escaped brackets
        // so it can be run against the regex and then substituted back in after the character positions
        // are calculated. I specifically picked two characters (\u{1} and \u{2}) because they are the
        // same width as a single bracket so the calculations will be correct, and they do not show up
        // as anything so they are unlikely to be used.
        let mut fmt_str = s
            .replace("{{", LEFT_PLACEHOLDER)
            .replace("}}", RIGHT_PLACEHOLDER);

        while let Some(mat) = spec::spec_regex_simple().captures_read_at(&mut locs, &fmt_str, pos) {
            let (start, end) = locs
                .get(0)
                .expect("Unable to get group 0 on CaptureLocations");
            spec_ranges.push(start..end);
            pos = end;
            let spec = FormatSpec::new(start - removed, spec_num, mat.as_str())?;
            spec_num += 1;
            removed += mat.as_str().len();
            specs.push(spec);
        }

        for range in spec_ranges.iter().rev() {
            fmt_str.replace_range(range.start..range.end, "");
        }

        let output = fmt_str
            .replace(LEFT_PLACEHOLDER, "{")
            .replace(RIGHT_PLACEHOLDER, "}");

        Ok((output, specs))
    }

    fn parse_args(args: &[String]) -> Vec<FormatArg> {
        args.iter()
            .enumerate()
            .map(|(n, a)| FormatArg::new(n, a))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
    // Lets try , , , and .
    #[test]
    fn generate() {
        const INPUT: &str = "Lets try {0}, {1}, {2}, and {}.";
        let f = Formatter::new(INPUT).unwrap();
        // println!("Formatter = {:#?}", f);
        let output = f.generate(&["one", "two", "three", "four"]).unwrap();
        // println!("Output = {}", output);
        assert_eq!(output, "Lets try one, two, three, and one.");
    }

    #[test]
    fn format() {
        const INPUT: &str = "Lets try {0}, {1}, {2}, and {}.";
        let output = Formatter::format(INPUT, &["one", "two", "three", "four"]).unwrap();
        // println!("Output = {}", output);
        assert_eq!(output, "Lets try one, two, three, and one.");
    }

    #[test]
    fn format_owned() {
        const INPUT: &str = "Let the {} beat {}.";
        let args = vec!["motherfucking", "drop"];
        let ref_args = args.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let output = Formatter::format(INPUT, &args).unwrap();
        // println!("Output = {}", output);
        assert_eq!(output, "Let the motherfucking beat drop.");
        let output = Formatter::format_owned(INPUT, &ref_args).unwrap();
        // println!("Output = {}", output);
        assert_eq!(output, "Let the motherfucking beat drop.");
    }

    #[test]
    fn escaped() {
        const INPUT: &str = "Hi {}, these are brackets: {{}}";
        const INPUT2: &str = "These brackets {{}} are super cool right {}?";
        let args = vec!["Tony"];
        let ref_args = args.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let output = Formatter::format(INPUT, &args).unwrap();
        // println!("Output = {}", output);
        assert_eq!(output, "Hi Tony, these are brackets: {}");
        let output = Formatter::format(INPUT2, &args).unwrap();
        // println!("Output = {}", output);
        assert_eq!(output, "These brackets {} are super cool right Tony?");
    }

    #[test]
    #[should_panic]
    fn bad_escape() {
        let _ = Formatter::new(format!("Here is my {} very bad string", "\u{1}").as_str());
    }

    #[test]
    fn weirdo1() {
        const INPUT: &str = "Thats {} too many {4} bro.";
        let args = vec!["way", "drop", "drop", "drop", "args"];
        let ref_args = args.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let output = Formatter::format(INPUT, &args).unwrap();
        // println!("Output = {}", output);
        assert_eq!(output, "Thats way too many args bro.");
        let output = Formatter::format_owned(INPUT, &ref_args).unwrap();
        // println!("Output = {}", output);
        assert_eq!(output, "Thats way too many args bro.");
        let f = Formatter::new(INPUT).unwrap();
        let output = f.generate(&args).unwrap();
        // println!("Output = {}", output);
        assert_eq!(output, "Thats way too many args bro.");
    }
}
