// Copyright (c) 2022 Tony Barbitta
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[derive(Debug, Default, Clone)]
pub struct FormatArg {
    pub pos: usize,
    pub name: Option<String>,
    pub value: String,
}

impl FormatArg {
    pub(crate) fn new(arg_position: usize, arg_text: &str) -> FormatArg {
        if let Some(eq) = arg_text.find('=') {
            let (name, rest) = arg_text.split_at(eq);
            let name = name.trim().to_string();
            let value = rest.trim_start_matches('=').trim().to_string();
            FormatArg {
                pos: arg_position,
                name: if name.is_empty() { None } else { Some(name) },
                value,
            }
        } else {
            FormatArg {
                pos: arg_position,
                name: None,
                value: arg_text.trim().to_string(),
            }
        }
    }

    pub fn is_named(&self, name: &str) -> bool {
        matches!(self.name, Some(ref n) if n == name)
    }

    pub fn is_pos(&self, pos: usize) -> bool {
        self.pos == pos
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn has_value(&self) -> bool {
        !self.value.is_empty()
    }
}

impl<T: std::fmt::Display> From<(usize, T)> for FormatArg {
    fn from((n, text): (usize, T)) -> Self {
        Self::new(n, text.to_string().as_str())
    }
}

#[derive(Debug, Default, Clone)]
pub struct FormatArgs(Vec<FormatArg>);

impl FormatArgs {
    pub fn new(input: Vec<FormatArg>) -> Self {
        let fa = Self(input);
        debug_assert!(fa.is_valid());
        fa
    }

    pub fn is_valid(&self) -> bool {
        // TODO: Should an empty `FormatArgs` be valid?
        if self.0.is_empty() {
            return true;
        }

        if self.0.iter().any(|fa| !fa.has_value()) {
            eprintln!("FormatArgs contains empty arg(s)");
            return false;
        }

        // Check that all positions exist
        let mut positions = self.0.iter().map(|fa| fa.pos).collect::<Vec<_>>();
        let pos_count = positions.len();
        positions.sort_unstable();
        positions.dedup();
        if positions.len() != pos_count {
            eprintln!("FormatArgs contains duplicate positions");
            return false;
        }
        for (i, pos) in positions.iter().enumerate() {
            if *pos != i {
                eprintln!("FormatArgs does not contain all sequential positions");
                return false;
            }
        }

        let mut names = self.0.iter().filter_map(|fa| fa.name()).collect::<Vec<_>>();
        let name_count = names.len();
        names.sort_unstable();
        names.dedup();
        if names.len() != name_count {
            eprintln!("FormatArgs contains duplicate names");
            return false;
        }

        true
    }

    pub fn empty() -> FormatArgs {
        Default::default()
    }

    pub fn push(&mut self, n: usize, a: &str) {
        self.0.push(FormatArg::new(n, a));
    }

    pub fn iter(&self) -> impl Iterator<Item = &FormatArg> {
        self.0.iter()
    }

    pub fn get_named(&self, name: &str) -> Option<&String> {
        self.iter()
            .find(|a| a.is_named(name))
            // .find(|a| a.name.as_ref().map(|n| n == name).unwrap_or(false))
            .map(|a| &a.value)
    }

    pub fn get(&self, pos: usize) -> Option<&String> {
        if self.is_empty() || pos > self.len() - 1 {
            return None;
        }

        self.iter().find(|a| a.is_pos(pos)).map(|a| &a.value)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn inner(&self) -> &Vec<FormatArg> {
        &self.0
    }
}

impl<T: Into<FormatArg>> FromIterator<T> for FormatArgs {
    fn from_iter<It: IntoIterator<Item = T>>(iter: It) -> Self {
        FormatArgs::new(iter.into_iter().map(Into::into).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

    #[test]
    fn arg_works() {
        let arg = FormatArg::new(0, "foobar");
        assert_eq!(arg.pos(), 0);
        assert_eq!(arg.name(), None);
        assert_eq!(arg.value(), "foobar");
        assert!(arg.has_value());

        let arg = FormatArg::new(0, "foo = bar");
        assert_eq!(arg.pos(), 0);
        assert_eq!(arg.name(), Some("foo"));
        assert_eq!(arg.value(), "bar");
        assert!(arg.has_value());

        let arg = FormatArg::new(0, "foo =");
        assert_eq!(arg.pos(), 0);
        assert_eq!(arg.name(), Some("foo"));
        assert_eq!(arg.value(), "");
        assert!(!arg.has_value());

        let arg = FormatArg::new(0, "= bar");
        assert_eq!(arg.pos(), 0);
        assert_eq!(arg.name(), None);
        assert_eq!(arg.value(), "bar");
        assert!(arg.has_value());
    }

    #[test]
    fn args_works() {
        let fargs = ["foobar", "foo = bar", "baz", "tig = old biddies"]
            .into_iter()
            .enumerate()
            .collect::<FormatArgs>();

        assert!(fargs.is_valid());
        assert_eq!(fargs.len(), 4);
        assert_eq!(fargs.get(0).expect("Unable to get(0)"), "foobar");
        assert_eq!(fargs.get(1).expect("Unable to get(1)"), "bar");
        assert_eq!(
            fargs.get_named("foo").expect("Unable to get_named(foo)"),
            "bar"
        );
        assert_eq!(
            fargs.get_named("tig").expect("Unable to get_named(tig)"),
            "old biddies"
        );
    }

    #[test]
    #[should_panic]
    fn args_catches_empty_value() {
        let _ = ["foobar", "foo = bar", "foo = ", "= bar"]
            .into_iter()
            .enumerate()
            .collect::<FormatArgs>();
    }
}
