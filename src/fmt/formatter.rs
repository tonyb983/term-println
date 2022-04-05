// Copyright (c) 2022 Tony Barbitta
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use unicode_width::UnicodeWidthStr;

use crate::{
    spec_regex_brackets_only as format_regex, Alignment, Error, FormatArg, FormatArgs, FormatSpec,
    Result,
};

#[derive(Debug, Clone)]
pub struct Formatter {
    expected_args: u8,
    fmt_str: String,
    fmt_spec: Vec<FormatSpec>,
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

    pub fn new(fmt_str: &str) -> crate::Result<Self> {
        let (s, spec) = match Self::parse_fmt(fmt_str) {
            Ok((s, spec)) => (s, spec),
            Err(err) => return Err(err),
        };

        // TODO: There seems like there should be an easier way to do this. Luckily most of these are copyable types
        //       and references, so it shouldnt be *too* expensive, but maybe still more than i'm comfortable with.
        // Expected args seems to be `max(A, B) + C` where
        //   A) the number of "bare" or "empty" args (aka "{}")
        //   B) the highest numbered positional arg
        //   C) the number of named args
        // As an example, using this println statement:
        /*
           println!(
               //                                      0                  1   2   3      4
               "I'm {tony}. Testing {0}, {1}, {2}, and {}. Again, that's {}, {}, {} and {}. What the {fk}?!",
               "one",
               "two",
               "three",
               "four",
               tony = "Tony",
               fk = "Whaaaaat"
           ); // Output: Tony, one, two, three, one, two, three, four, Tony, Whaaaaat
           Interesting to note, that the named argument 'tony' gets put into the positional argument #4
        */
        // The named arguments ("tony" and "fk") will never consume a non-named arg, while the positional args will.
        // As such, this does not work: println!("Testing {0}, {1}, {2} and {}", "one", "two", "three", "four");
        // So if we have println!("{0} {1} {2} {3}")
        let empty_args = spec.iter().filter(|s| s.is_empty()).count();
        let highest_pos = spec.iter().filter_map(|s| s.arg_num).max().unwrap_or(0);
        let mut all_names = spec
            .iter()
            .filter_map(|s| s.arg_name.as_deref())
            .collect::<Vec<_>>();
        all_names.sort_unstable();
        all_names.dedup();
        let unique_names = all_names.len();

        let expected = (empty_args.max(highest_pos) + unique_names) as u8;
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
        // let args = args.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let args: FormatArgs = args.iter().enumerate().collect();
        let mut positional_count = 0usize;
        // Unused at the moment, since we iterate in the ranges in reverse, we no longer need to track character offset
        let mut offset = 0usize;
        let mut mods = Vec::new();

        for spec in &self.fmt_spec {
            let insert = if let Some(num) = spec.arg_num {
                match args.get(num) {
                    Some(s) => s,
                    None => {
                        eprintln!("Unable to find numbered arg #{}", num);
                        return Err(crate::Error::bad_arg_num(num, args.len()));
                    }
                }
            } else if let Some(ref name) = spec.arg_name {
                match args.get_named(name) {
                    Some(s) => s,
                    None => {
                        eprintln!("Unable to find named arg '{}'", name);
                        return Err(crate::Error::bad_arg_name(name));
                    }
                }
            } else {
                let s = match args.get(positional_count) {
                    Some(s) => s,
                    None => {
                        eprintln!("Positional arg requests have surpassed provided args");
                        return Err(crate::Error::bad_arg_num(positional_count, args.len()));
                    }
                };
                positional_count += 1;
                s
            };

            let width = match spec.width {
                Some(w) => w,
                None => UnicodeWidthStr::width(insert.as_str()),
            };
            let align = match spec.align {
                Some(a) => a,
                None => Alignment::Left,
            };
            let prepared = Self::prepare_string(insert.as_str(), align, width);

            mods.push((prepared, spec.fmt_pos));
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

    pub fn prepare_string(s: &str, align: Alignment, width: usize) -> String {
        let str_size = UnicodeWidthStr::width(s);
        if str_size == width {
            return s.to_string();
        }

        let mut output = String::with_capacity(width);

        if width > str_size {
            let pad_char = ' ';
            let pad_count = width - str_size;
            match align {
                Alignment::Left => {
                    output.push_str(s);
                    output.extend(std::iter::repeat(pad_char).take(pad_count));
                }
                Alignment::Center => {
                    let left_pad = pad_count / 2;
                    let right_pad = pad_count - left_pad;
                    output.extend(std::iter::repeat(pad_char).take(left_pad));
                    output.push_str(s);
                    output.extend(std::iter::repeat(pad_char).take(right_pad));
                }
                Alignment::Right => {
                    output.extend(std::iter::repeat(pad_char).take(pad_count));
                    output.push_str(s);
                }
            }
        } else {
            match align {
                Alignment::Left => {
                    let uni_width = if s.is_char_boundary(width) {
                        width
                    } else {
                        s.floor_char_boundary(width)
                    };
                    let trimmed = &s[..uni_width];
                    output.push_str(trimmed);
                }
                Alignment::Center => {
                    let diff = str_size - width;
                    let left = diff / 2;
                    let right = diff - left;
                    let start = if s.is_char_boundary(left) {
                        left
                    } else {
                        s.floor_char_boundary(left)
                    };
                    let end = if s.is_char_boundary(str_size - right) {
                        str_size - right
                    } else {
                        s.floor_char_boundary(str_size - right)
                    };
                    let trimmed = &s[start..end];
                    output.push_str(trimmed);
                }
                Alignment::Right => {
                    let start = str_size - width;
                    let uni_start = if s.is_char_boundary(start) {
                        start
                    } else {
                        s.ceil_char_boundary(start)
                    };
                    let trimmed = &s[uni_start..];
                    output.push_str(trimmed);
                }
            }
        }

        output
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
        // "\u{1}" - Unknown, but length 1
        // "\u{2}" - Unknown, but length 1
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

        let mut locs = format_regex().capture_locations();
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

        while let Some(mat) = format_regex().captures_read_at(&mut locs, &fmt_str, pos) {
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

    fn parse_args(args: &[String]) -> FormatArgs {
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

    #[test]
    fn chars() {
        fn print_and_len<S: AsRef<str>>(input: S) {
            let input = input.as_ref();
            println!("Length of '{}' is {}", input, input.len());
        }

        let bl = "{";
        let br = "}";
        let bbl = "{{";
        let bbr = "}}";
        let uni = "\u{F0000}";
        let uni2 = "\u{AE}";
        let uni3 = "\u{0}";
        let uni4 = "\u{1}";
        let uni5 = "\u{2}";
        print_and_len(bl);
        print_and_len(bbl);
        print_and_len(br);
        print_and_len(bbr);
        print_and_len(uni);
        print_and_len(uni2);
        print_and_len(uni3);
        print_and_len(uni4);
        print_and_len(uni5);
        print_and_len("‰");
    }

    #[test]
    fn prepare_string() {
        let string = "0123456789";
        let left20 = Formatter::prepare_string(string, Alignment::Left, 20);
        let mid20 = Formatter::prepare_string(string, Alignment::Center, 20);
        let right20 = Formatter::prepare_string(string, Alignment::Right, 20);
        assert_eq!(left20, "0123456789          ");
        assert_eq!(mid20, "     0123456789     ");
        assert_eq!(right20, "          0123456789");
        let left8 = Formatter::prepare_string(string, Alignment::Left, 8);
        let mid8 = Formatter::prepare_string(string, Alignment::Center, 8);
        let right8 = Formatter::prepare_string(string, Alignment::Right, 8);
        assert_eq!(left8, "01234567");
        assert_eq!(mid8, "12345678");
        assert_eq!(right8, "23456789");
        let left5 = Formatter::prepare_string(string, Alignment::Left, 5);
        let mid5 = Formatter::prepare_string(string, Alignment::Center, 5);
        let right5 = Formatter::prepare_string(string, Alignment::Right, 5);
        assert_eq!(left5, "01234");
        assert_eq!(mid5, "23456");
        assert_eq!(right5, "56789");

        //                   1234
        let chinese = "读文读文";
        assert_eq!(UnicodeWidthStr::width(chinese), 8);
        let left4 = Formatter::prepare_string(chinese, Alignment::Left, 4);
        let mid4 = Formatter::prepare_string(chinese, Alignment::Center, 4);
        let right4 = Formatter::prepare_string(chinese, Alignment::Right, 4);
        // These are all sorts of jacked up due to char byte boundaries :shrug:
        assert_eq!(left4, "读");
        assert_eq!(mid4, "读文");
        assert_eq!(right4, "读文");

        //                 " 1234567890123456"
        let hearts = "💜💙💚💛💚💙💜";
        // ???????
        assert_eq!(UnicodeWidthStr::width("❤️"), 1);
        assert_eq!(UnicodeWidthStr::width("🧡"), 2);
        assert_eq!(UnicodeWidthStr::width("💛"), 2);
        assert_eq!(UnicodeWidthStr::width("💚"), 2);
        assert_eq!(UnicodeWidthStr::width("💙"), 2);
        assert_eq!(UnicodeWidthStr::width("💜"), 2);
        // ??????????
        assert_eq!(UnicodeWidthStr::width(hearts), 14);
        // Unicode makes literally zero fucking sense
        let left8 = Formatter::prepare_string(hearts, Alignment::Left, 8);
        assert_eq!(left8, "💜💙");
    }
}
