//! Autogenerate Markdown documentation for clap command-line tools
//!
//! See [**Examples**][Examples] for examples of the content `clap-markdown`
//! generates.
//!
//! [Examples]: https://github.com/ConnorGray/clap-markdown#Examples
//!

#[allow(dead_code)]


// Ensure that doc tests in the README.md file get run.
#[doc(hidden)]
mod test_readme {
    #![doc = include_str!("../README.md")]
}


use std::fmt::{self, Write};

use clap::{builder::PossibleValue, Arg, Command};

/// Format the help information for `command` as Markdown.
pub fn help_markdown<C: clap::CommandFactory>() -> String {
    let command = C::command();

    help_markdown_command(&command)
}

/// Format the help information for `command` as Markdown.
pub fn help_markdown_command(command: &clap::Command) -> String {
    let mut buffer = String::with_capacity(100);

    write_help_markdown(&mut buffer, command);

    buffer
}

//======================================
// Markdown
//======================================

/// Format the help information for `command` as Markdown and print it.
///
/// Output is printed to the standard output, using [`println!`].
pub fn print_help_markdown<C: clap::CommandFactory>() {
    let command = C::command();

    let mut buffer = String::with_capacity(100);

    write_help_markdown(&mut buffer, &command);

    println!("{}", buffer);
}

fn write_help_markdown(buffer: &mut String, command: &clap::Command) {
    //----------------------------------
    // Write the document title
    //----------------------------------

    let title_name = match command.get_display_name() {
        Some(display_name) => display_name.to_owned(),
        None => format!("`{}`", command.get_name()),
    };

    writeln!(buffer, "# Command-Line Help for {title_name}\n").unwrap();

    writeln!(
        buffer,
        "This document contains the help content for the `{}` command-line program.\n",
        command.get_name()
    ).unwrap();

    //----------------------------------
    // Write the table of contents
    //----------------------------------

    // writeln!(buffer, r#"<div style="background: light-gray"><ul>"#).unwrap();
    // build_table_of_contents_html(buffer, Vec::new(), command, 0).unwrap();
    // writeln!(buffer, "</ul></div>").unwrap();

    writeln!(buffer, "**Command Overview:**\n").unwrap();

    build_table_of_contents_markdown(buffer, Vec::new(), command, 0).unwrap();

    write!(buffer, "\n").unwrap();

    //----------------------------------------
    // Write the commands/subcommands sections
    //----------------------------------------

    // build_command_markdown(buffer, Vec::new(), command, 0).unwrap();
    build_cmd(&command);

    //-----------------
    // Write the footer
    //-----------------

//     write!(buffer, r#"<hr/>

// <small><i>
//     This document was generated automatically by
//     <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
// </i></small>
// "#).unwrap();
}


struct HlpCmd {
    name: String,
    long_flag: String,
    short_flag: String,

    long_about: String,
    short_about: String,

    length: usize,
    options: Vec<HlpArg>,
    commands: Vec<HlpCmd>,
}

impl HlpCmd {
    fn new(cmd: &Command) -> Self {
        let short_about = match cmd.get_about() {
            Some(value) => value.to_string(),
            None => String::new(),
        };

        let long_about = match cmd.get_long_about() {
            Some(value) => value.to_string(),
            None => String::new(),
        };

        let long_flag = match cmd.get_long_flag() {
            Some(value) => value.to_string(),
            None => String::new(),
        };

        let short_flag = match cmd.get_short_flag() {
            Some(value) => value.to_string(),
            None => String::new(),
        };

        HlpCmd {
            name: cmd.get_name().to_string(),
            long_flag,
            short_flag,
            long_about,
            short_about,
            length: 0,
            options: Vec::new(),
            commands: Vec::new(),
        }
    }

    fn max_len(&mut self, i: usize) -> &Self {
        if i > self.length {
            self.length = i;
        }
        self
    }

    fn add_arg(&mut self, arg: HlpArg) -> &Self {
        let size = &arg.long.len();
        self.options.push(arg);
        self.max_len(*size);
        self
    }

    fn add_cmd(&mut self, cmd: HlpCmd) -> &Self {
        self.commands.push(cmd);
        self
    }

    fn print(&self) {
        println!("{}", &self.short_about);
        println!("\n");

        self.print_commands();

        if !self.options.is_empty() {
            println!("Options");
            for arg in self.options.iter() {
                arg.print(self.length);
            }
            println!("\n");
        }
    }

    fn print_commands(&self) {
        if self.commands.is_empty() {
            return;
        }

        let mut width: usize = 0;
        for cmd in self.commands.iter() {
            let name_length = match cmd.name.len() {
                0 => 0,
                value => value + 2, // value + comma + space
            };

            let short_length = match cmd.short_flag.len() {
                0 => 0,
                value => value + 3, // value + single-dash + comma + space
            };

            let long_length = match cmd.long_flag.len() {
                0 => 0,
                value => value + 3 // value + double-dash + comma
            };

            width = std::cmp::max(width, name_length + short_length + long_length);
        }
        width += 2; // 2 trailing spaces

        println!("Commands");

        let mut f = String::new();
        for cmd in self.commands.iter() {
            let name = match &cmd.name.is_empty() {
                false => format!("{}, ", cmd.name),
                true => "".to_string(),
            };

            let short_flag = match &cmd.short_flag.is_empty() {
                false => format!("-{}, ", cmd.short_flag),
                true => "".to_string(),
            };

            let long_flag = match &cmd.long_flag.is_empty() {
                false => format!("--{}", cmd.long_flag),
                true => "".to_string(),
            };

            writeln!(f, "{:width$} {}", format!("  {}{}{}", name, short_flag, long_flag), &cmd.short_about).unwrap();
        }
        println!("{}", f);
        println!("\n");

    }
}

#[derive(Clone)]
struct HlpArg {
    short: char,
    long: String,
    value: String,
    description: String,
}

impl HlpArg {
    fn new(arg: &Arg) -> Self {
        let short = match arg.get_short() {
            Some(value) => value,
            None => ' ',
        };
        let long = match arg.get_long() {
            Some(value) => value.to_string(),
            None => String::new(),
        };
        let value = match arg.get_action().takes_values() {
            true =>  match arg.get_value_names() {
                // TODO: What if multiple names are provided?
                Some([]) => unreachable!(
                    "clap Arg::get_value_names() returned Some(..) of empty list"
                ),
                Some(value) => value.join("|"),
                None => arg.get_id().to_string().to_ascii_uppercase(),
            }
            false => String::new(),
        };

        let description = match arg.get_help() {
            Some(value) => value.to_string(),
            None => String::new(),
        };

        HlpArg {
            short,
            long,
            value,
            description,
        }
    }

    fn print(&self, length: usize) {
        let mut s = String::new();
        match self.short {
            ' ' => write!(s, "      "),
            value => write!(s, "  -{}, ", value),
        }.expect("error");

        let width = length + 2;
        write!(s, "--{}", format!("{:width$}", self.long)).expect("error");

        write!(s, "<{}>  ", self.value).expect("error");

        write!(s, "{}", self.description).expect("error");
        println!("{}", s);
    }
}


fn build_cmd(command: &Command) -> HlpCmd {
  let mut hc = HlpCmd::new(&command);

  for pos_arg in command.get_positionals() {
    let short = pos_arg.get_long();
    let long = pos_arg.get_long();
    let desc = pos_arg.get_help();
    println!("-{:#?}\t --{:#?}\t Short: {:#?}", short, long, desc);
  }

  for cmd in command.get_subcommands() {
    hc.add_cmd(HlpCmd::new(&cmd));
  }

  for pos_arg in command.get_arguments() {
    hc.add_arg(HlpArg::new(&pos_arg));
  }

  hc.print();

  hc
}



fn build_table_of_contents_markdown(
    buffer: &mut String,
    // Parent commands of `command`.
    parent_command_path: Vec<String>,
    command: &clap::Command,
    depth: usize,
) -> std::fmt::Result {
    // Don't document commands marked with `clap(hide = true)` (which includes
    // `print-all-help`).
    if command.is_hide_set() {
        return Ok(());
    }

    // Append the name of `command` to `command_path`.
    let command_path = {
        let mut command_path = parent_command_path;
        command_path.push(command.get_name().to_owned());
        command_path
    };

    writeln!(
        buffer,
        "* [`{}`↴](#{})",
        command_path.join(" "),
        command_path.join("-"),
    )?;

    //----------------------------------
    // Recurse to write subcommands
    //----------------------------------

    for subcommand in command.get_subcommands() {
        build_table_of_contents_markdown(
            buffer,
            command_path.clone(),
            subcommand,
            depth + 1,
        )?;
    }

    Ok(())
}

/*
fn build_table_of_contents_html(
    buffer: &mut String,
    // Parent commands of `command`.
    parent_command_path: Vec<String>,
    command: &clap::Command,
    depth: usize,
) -> std::fmt::Result {
    // Don't document commands marked with `clap(hide = true)` (which includes
    // `print-all-help`).
    if command.is_hide_set() {
        return Ok(());
    }

    // Append the name of `command` to `command_path`.
    let command_path = {
        let mut command_path = parent_command_path;
        command_path.push(command.get_name().to_owned());
        command_path
    };

    writeln!(
        buffer,
        "<li><a href=\"#{}\"><code>{}</code>↴</a></li>",
        command_path.join("-"),
        command_path.join(" ")
    )?;

    //----------------------------------
    // Recurse to write subcommands
    //----------------------------------

    for subcommand in command.get_subcommands() {
        build_table_of_contents_html(
            buffer,
            command_path.clone(),
            subcommand,
            depth + 1,
        )?;
    }

    Ok(())
}
*/

#[allow(dead_code)]
fn build_command_markdown(
    buffer: &mut String,
    // Parent commands of `command`.
    parent_command_path: Vec<String>,
    command: &clap::Command,
    depth: usize,
) -> std::fmt::Result {
    // Don't document commands marked with `clap(hide = true)` (which includes
    // `print-all-help`).
    if command.is_hide_set() {
        return Ok(());
    }

    // Append the name of `command` to `command_path`.
    let command_path = {
        let mut command_path = parent_command_path.clone();
        command_path.push(command.get_name().to_owned());
        command_path
    };

    //----------------------------------
    // Write the markdown heading
    //----------------------------------

    // TODO: `depth` is now unused. Remove if no other use for it appears.
    /*
    if depth >= 6 {
        panic!(
            "command path nesting depth is deeper than maximum markdown header depth: `{}`",
            command_path.join(" ")
        )
    }
    */

    writeln!(
        buffer,
        "{} `{}`\n",
        // "#".repeat(depth + 1),
        "##",
        command_path.join(" "),
    )?;

    if let Some(long_about) = command.get_long_about() {
        writeln!(buffer, "{}\n", long_about)?;
    } else if let Some(about) = command.get_about() {
        writeln!(buffer, "{}\n", about)?;
    }

    // TODO(feature): Support printing custom before and after help texts.
    assert!(command.get_before_help().is_none());
    assert!(command.get_after_help().is_none());

    writeln!(
        buffer,
        "**Usage:** `{}{}`\n",
        if parent_command_path.is_empty() {
            String::new()
        } else {
            let mut s = parent_command_path.join(" ");
            s.push_str(" ");
            s
        },
        command
            .clone()
            .render_usage()
            .to_string()
            .replace("Usage: ", "")
    )?;

    //----------------------------------
    // Subcommands
    //----------------------------------

    if command.get_subcommands().next().is_some() {
        writeln!(buffer, "###### **Subcommands:**\n")?;

        for subcommand in command.get_subcommands() {
            if subcommand.is_hide_set() {
                continue;
            }

            writeln!(
                buffer,
                "* `{}` — {}",
                subcommand.get_name(),
                match subcommand.get_about() {
                    Some(about) => about.to_string(),
                    None => String::new(),
                }
            )?;
        }

        write!(buffer, "\n")?;
    }

    //----------------------------------
    // Arguments
    //----------------------------------

    if command.get_positionals().next().is_some() {
        writeln!(buffer, "###### **Arguments:**\n")?;

        for pos_arg in command.get_positionals() {
            write_arg_markdown(buffer, pos_arg)?;
        }

        write!(buffer, "\n")?;
    }

    //----------------------------------
    // Options
    //----------------------------------

    let non_pos: Vec<_> = command
        .get_arguments()
        .filter(|arg| !arg.is_positional())
        .collect();

    if !non_pos.is_empty() {
        writeln!(buffer, "###### **Options:**\n")?;

        for arg in non_pos {
            write_arg_markdown(buffer, arg)?;
        }

        write!(buffer, "\n")?;
    }

    //----------------------------------
    // Recurse to write subcommands
    //----------------------------------

    // Include extra space between commands. This is purely for the benefit of
    // anyone reading the source .md file.
    write!(buffer, "\n\n")?;

    for subcommand in command.get_subcommands() {
        build_command_markdown(
            buffer,
            command_path.clone(),
            subcommand,
            depth + 1,
        )?;
    }

    Ok(())
}

fn write_arg_markdown(buffer: &mut String, arg: &clap::Arg) -> fmt::Result {
    // Markdown list item
    write!(buffer, "* ")?;

    let value_name: String = match arg.get_value_names() {
        // TODO: What if multiple names are provided?
        Some([name, ..]) => name.as_str().to_owned(),
        Some([]) => unreachable!(
            "clap Arg::get_value_names() returned Some(..) of empty list"
        ),
        None => arg.get_id().to_string().to_ascii_uppercase(),
    };

    match (arg.get_short(), arg.get_long()) {
        (Some(short), Some(long)) => {
            if arg.get_action().takes_values() {
                write!(buffer, "`-{short}`, `--{long} <{value_name}>`")?
            } else {
                write!(buffer, "`-{short}`, `--{long}`")?
            }
        },
        (Some(short), None) => {
            if arg.get_action().takes_values() {
                write!(buffer, "`-{short} <{value_name}>`")?
            } else {
                write!(buffer, "`-{short}`")?
            }
        },
        (None, Some(long)) => {
            if arg.get_action().takes_values() {
                write!(buffer, "`--{} <{value_name}>`", long)?
            } else {
                write!(buffer, "`--{}`", long)?
            }
        },
        (None, None) => {
            debug_assert!(arg.is_positional(), "unexpected non-positional Arg with neither short nor long name: {arg:?}");

            write!(buffer, "`<{value_name}>`",)?;
        },
    }

    if let Some(help) = arg.get_help() {
        writeln!(buffer, " — {help}")?;
    } else {
        writeln!(buffer)?;
    }

    //--------------------
    // Arg default values
    //--------------------

    if !arg.get_default_values().is_empty() {
        let default_values: String = arg
            .get_default_values()
            .iter()
            .map(|value| format!("`{}`", value.to_string_lossy()))
            .collect::<Vec<String>>()
            .join(", ");

        if arg.get_default_values().len() > 1 {
            // Plural
            writeln!(buffer, "\n  Default values: {default_values}")?;
        } else {
            // Singular
            writeln!(buffer, "\n  Default value: {default_values}")?;
        }
    }

    //--------------------
    // Arg possible values
    //--------------------

    let possible_values: Vec<PossibleValue> = arg
        .get_possible_values()
        .into_iter()
        .filter(|pv| !pv.is_hide_set())
        .collect();

    if !possible_values.is_empty() {
        let any_have_help: bool =
            possible_values.iter().any(|pv| pv.get_help().is_some());

        if any_have_help {
            // If any of the possible values have help text, print them
            // as a separate item in a bulleted list, and include the
            // help text for those that have it. E.g.:
            //
            //     Possible values:
            //     - `value1`:
            //       The help text
            //     - `value2`
            //     - `value3`:
            //       The help text

            let text: String = possible_values
                .iter()
                .map(|pv| match pv.get_help() {
                    Some(help) => {
                        format!("  - `{}`:\n    {}\n", pv.get_name(), help)
                    },
                    None => format!("  - `{}`\n", pv.get_name()),
                })
                .collect::<Vec<String>>()
                .join("");

            writeln!(buffer, "\n  Possible values:\n{text}")?;
        } else {
            // If none of the possible values have any documentation, print
            // them all inline on a single line.
            let text: String = possible_values
                .iter()
                // TODO: Show PossibleValue::get_help(), and PossibleValue::get_name_and_aliases().
                .map(|pv| format!("`{}`", pv.get_name()))
                .collect::<Vec<String>>()
                .join(", ");

            writeln!(buffer, "\n  Possible values: {text}\n")?;
        }
    }

    Ok(())
}