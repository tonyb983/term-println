# FMT

A simple little utility that allows you to use rust-like println! syntax from the terminal. I made this because I needed something to distract me and I was quite tired of using printf and echo in my WSL fish shell.

Notable Sections
- [FormatSpec::new](./src/fmt/spec.rs:80) - Parses the individual format specifier
- [Formatter::parse_fmt](./src/fmt/mod.rs:93) - Parses the format string
- [Formatter::generate](./src/fmt/mod.rs:53) - Creates the output `String` by substituting args for placeholders
- [FormatArg::new](./src/fmt/arg.rs:15) - Does some minor parsing of the input arguments (basically it only checks for an equals sign, and if it is present, assigns the argument a name as well as a value)
- [Formatter::format](./src/fmt/mod.rs) & [Formatter::format_owned](./src/fmt/mod.rs) - Convenience functions that wrap `Formatter::new` and `Formatter::generate`, creating output from a format string and arguments

Todo
- [ ] Implement more formatting specs. I am currently parsing alignment and width but not using them at all.
- [ ] Investigate whether the unicode character substitution I use in [Formatter::parse_fmt](./src/fmt/mod.rs:89) is at all safe to do.
- [ ] Colors? Maybe add color as an option within the formatting spec?
- [ ] When implementing the alignment and width, use the `terminal_size` crate to make sure everything fits nicely.
- [ ] Clean up stuff, write more tests, the usual.