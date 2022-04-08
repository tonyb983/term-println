use ansirs::*;

pub fn print_usage(bin: &str) -> crate::Result<()> {
    fn header(text: &str) {
        println!("{}:", style_text(text, Ansi::from_fg(Colors::Yellow)));
    }
    fn term(cmd: &str, args: &[&str], indent: bool, quote_args: bool) {
        if args.is_empty() {
            println!(
                "{mt}{i} {c}",
                c = style_text(cmd, Ansi::from_fg(Colors::LawnGreen)),
                mt = if indent { "\t" } else { "" },
                i = style_text("$", Ansi::from_fg(Colors::GoldenRod))
            );
        } else {
            println!(
                "{mt}{i} {c} {a}",
                c = style_text(cmd, Ansi::from_fg(Colors::LawnGreen)),
                a = args
                    .iter()
                    .map(|s| if quote_args {
                        format!(
                            "{q}{}{q}",
                            if s.starts_with('-') {
                                style_text(s, Ansi::from_fg(Colors::Purple))
                            } else {
                                style_text(s, Ansi::from_fg(Colors::White))
                            },
                            q = style_text("\"", Ansi::from_fg(Colors::Gray))
                        )
                    } else {
                        style_text(s.to_string(), Ansi::from_fg(Colors::White))
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
                i = style_text("$", Ansi::from_fg(Colors::GoldenRod)),
                mt = if indent { "\t" } else { "" },
            );
        }
    }
    let this_bin = if let Some(n) = bin.rfind(['/', '\\']) {
        &bin[n + 1..]
    } else {
        bin
    };
    println!();
    header("Usage");
    term(
        this_bin,
        &["[FLAGS]", "<FMT_STRING>", "[<ARGS>]"],
        true,
        false,
    );
    println!();
    Ok(())
}

pub fn print_usage_long(bin: &str) -> crate::Result<()> {
    const TEXT_SPACE: usize = 16;
    fn header(text: &str) {
        println!("{}:", text);
    }
    fn subheader(text: &str) {
        println!("  {}:", text);
    }
    fn item_and_desc(item: &str, desc: &str) {
        println!("\t{:<2$}\t{}", item, desc, TEXT_SPACE);
    }
    fn term(cmd: &str, args: &[&str], indent: bool, quote_args: bool) {
        if args.is_empty() {
            println!(
                "{mt}{i} {c}",
                c = style_text(cmd, Ansi::from_fg(Colors::LawnGreen)),
                mt = if indent { "\t" } else { "" },
                i = style_text("$", Ansi::from_fg(Colors::GoldenRod))
            );
        } else {
            println!(
                "{mt}{i} {c} {a}",
                c = style_text(cmd, Ansi::from_fg(Colors::LawnGreen)),
                a = args
                    .iter()
                    .map(|s| if quote_args {
                        format!(
                            "{q}{}{q}",
                            if s.starts_with('-') {
                                style_text(s, Ansi::from_fg(Colors::Purple))
                            } else {
                                style_text(s, Ansi::from_fg(Colors::White))
                            },
                            q = style_text("\"", Ansi::from_fg(Colors::Gray))
                        )
                    } else {
                        style_text(s.to_string(), Ansi::from_fg(Colors::White))
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
                i = style_text("$", Ansi::from_fg(Colors::GoldenRod)),
                mt = if indent { "\t" } else { "" },
            );
        }
    }
    fn term_out(text: &str, indent: bool) {
        println!(
            "{mt}{i} {0}",
            style_text(text, Ansi::from_fg(Colors::White)),
            mt = if indent { "\t" } else { "" },
            i = style_text("$", Ansi::from_fg(Colors::GoldenRod))
        );
    }

    let this_bin = if let Some(n) = bin.rfind(['/', '\\']) {
        &bin[n + 1..]
    } else {
        bin
    };
    // Main usage
    header("Usage");
    term(
        this_bin,
        &["[FLAGS]", "<FMT_STRING>", "[<ARGS>]"],
        true,
        false,
    );
    println!();
    // Argument description
    header("Arguments");
    item_and_desc(
        "FMT_STRING",
        "A string containing text and any number of FMT_SPECs (format specifiers, see below)",
    );
    item_and_desc(
        "ARGS",
        "A list of strings to be inserted into the FMT_STRING",
    );
    println!();
    // Flag description
    header("Flags");
    item_and_desc("-h, --help", "Print this help message and exit immediately");
    item_and_desc(
        "-D, --debug",
        "Print debug information while parsing the FMT_STRING and ARGS",
    );
    println!();
    // Format specifier details
    header("Format specifiers");
    item_and_desc(
        "{}",
        "The most basic specifier, will substitute ARGS unchanged in order of appearance",
    );
    item_and_desc(
        "{0}, .., {n}",
        "Numbered specifier, corresponding to ARGS in order of appearance, zero indexed",
    );
    item_and_desc(
        "{name}",
        "Named specifier, corresponding to ARGS in the form of \"name = value\"",
    );
    item_and_desc(
        "{:5}, {:10}, {:n}",
        "Width specifier, dictates how much space the ARG will occupy",
    );
    item_and_desc(
        "{:<}, {:^}, {:>}",
        "Alignment specifier, aligns ARG to the left, center, or right (useless without width)",
    );
    println!();

    // Usages Examples
    header("Examples");

    subheader("Basic");
    term(this_bin, &["Number {}!", "1"], true, true);
    term_out("Number 1!", true);

    subheader("Numbered");
    term(
        this_bin,
        &["Number {1} and Number {0}!", "2", "1"],
        true,
        true,
    );
    term_out("Number 1 and Number 2!", true);
    subheader("Numbered (Ridiculous)");
    term(
        this_bin,
        &[
            "Number {1} and Number {9}!",
            "0",
            "1",
            "2",
            "3",
            "4",
            "5",
            "6",
            "7",
            "8",
            "9",
        ],
        true,
        true,
    );
    term_out("Number 1 and Number 9!", true);

    subheader("Named");
    term(
        this_bin,
        &["Number {n} and Number {}!", "2", "n = 1"],
        true,
        true,
    );
    term_out("Number 1 and Number 2!", true);

    subheader("Width");
    term(
        this_bin,
        &["Number |{:5}| and Number |{1:10}|!", "1", "2"],
        true,
        true,
    );
    term_out("Number |    1| and Number |         2|!", true);

    subheader("Alignment");
    term(
        this_bin,
        &[
            "Number |{1:<5}| and |{two:^5}| and |{0:>5}|!",
            "3",
            "1",
            "two = 2",
        ],
        true,
        true,
    );
    term_out("Number |1    | and |  2  | and |    3|!", true);

    Ok(())
}
