// Copyright (c) 2022 Tony Barbitta
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub struct FormatArg {
    pub pos: usize,
    pub name: Option<String>,
    pub value: String,
}

impl FormatArg {
    pub(crate) fn new(n: usize, a: &str) -> FormatArg {
        if let Some(eq) = a.find('=') {
            let (name, rest) = a.split_at(eq);
            let name = name.trim().to_string();
            let value = rest.trim_start_matches('=').trim().to_string();
            FormatArg {
                pos: n,
                name: Some(name),
                value,
            }
        } else {
            FormatArg {
                pos: n,
                name: None,
                value: a.to_string(),
            }
        }
    }
}
