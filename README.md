# FMT

A simple little utility that allows you to use rust-like println! syntax from the terminal. I made this because I needed something to distract me and I was quite tired of using printf and echo in my WSL fish shell.

Todo
- [ ] Implement more formatting specs. I am currently parsing alignment and width but not using them at all.
- [ ] Investigate whether the unicode character substitution I use in [Formatter::parse_fmt](./src/fmt/mod.rs:89) is at all safe to do.
- [ ] Colors? Maybe add color as an option within the formatting spec?
- [ ] When implementing the alignment and width, use the `terminal_size` crate to make sure everything fits nicely.
- [ ] Clean up stuff, write more tests, the usual.