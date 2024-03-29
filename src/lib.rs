//! Generate documentation for clap command-line tools

// static TEMPLATE_FILE: &'static [u8] = include_bytes!("./test.html");

#[allow(dead_code)]
// Ensure that doc tests in the README.md file get run.
#[doc(hidden)]
mod test_readme {
    #![doc = include_str!("../README.md")]
}

use std::fmt::{Arguments, Write};

use clap::{Arg, Command};
use handlebars::Handlebars;
use serde_derive::Serialize;

#[derive(Serialize, Clone, Debug)]
struct FmtArg {
    flags: String,
    description: String
}

#[derive(Serialize, Clone, Debug)]
struct FmtCmd {
    name: String,
    description: String,
}


#[derive(Serialize, Clone, Debug)]
struct FmtCommands {
    title: String,
    usage: String,
    cmd_chain: String,
    description: String,
    commands: Vec<FmtCmd>,
    arguments: Vec<FmtArg>,
    options: Vec<FmtArg>,
}

#[derive(Serialize, Clone, Debug)]
struct Page {
    main: FmtCommands,
    subcommands: Vec<FmtCommands>
}

/// Format the help information for `command` as Markdown.
pub fn write_help_factory<C: clap::CommandFactory>() -> String {
    let command = C::command();

    help_command(&command)
}

/// Format the help information for `command` as Markdown.
pub fn help_command(command: &clap::Command) -> String {
    let mut buffer = String::with_capacity(100);

    write_help(&mut buffer, command);

    buffer
}

/// Format the help information for `command` as Markdown and print it.
///
/// Output is printed to the standard output, using [`println!`].
pub fn print_help<C: clap::CommandFactory>() {
    let command = C::command();

    let mut buffer = String::with_capacity(100);

    write_help(&mut buffer, &command);

    println!("{}", buffer);
}

fn write_help(buffer: &mut String, command: &clap::Command) {
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
    )
    .unwrap();

    build_cmd(&command);
}

trait Cmd {
    fn fmt_cmd(&self, parent: Vec<String>) -> FmtCommands;
    fn get_usage(&mut self) -> String;
}

impl Cmd for Command {
    fn get_usage(&mut self) -> String {
        let parts = self.render_usage().to_string();
        let parts = parts.split(" ").collect::<Vec<&str>>();
        let slice = parts[2..].iter().map(|v| v.to_string()).collect::<Vec<String>>();
        slice.join(" ").to_string()
    }

    fn fmt_cmd(&self, parents: Vec<String>) -> FmtCommands {

        // TODO: Get the actual long about
        let long_about = match self.get_about() {
            Some(value) => value.to_string(),
            None => String::new(),
        };

        let mut arguments: Vec<FmtArg> = Vec::new();
        let mut options: Vec<FmtArg> = Vec::new();
        for arg in self.get_arguments() {

            if arg.is_positional() {
                arguments.push(FmtArg {
                    flags: arg.fmt_flags(),
                    description: match arg.get_long_help() {
                        Some(value) => value.to_string(),
                        None => String::new(),
                    }
                });
            } else {
                options.push(FmtArg {
                    flags: arg.fmt_flags(),
                    description: match arg.get_long_help() {
                        Some(value) => value.to_string(),
                        None => String::new(),
                    }
                });
            }
        }

        // Format the subcommands
        let mut subcommands: Vec<FmtCmd> = Vec::new();
        for command in self.get_subcommands() {
            subcommands.push(FmtCmd {
                name: command.get_name().to_string(),
                description: match command.get_after_long_help() {
                    Some(t) => t.to_string(),
                    _ => String::new(),
                }
            });
        }

        let mut cmd = self.clone();
        let usage = cmd.get_usage();

        let mut ancestors = parents.clone();
        ancestors.push(self.get_name().to_string());

        FmtCommands {
            title: self.get_name().to_string(),
            usage,
            cmd_chain: ancestors.join(" "),
            description: long_about,
            commands: subcommands,
            arguments,
            options,
        }
    }
}



/**
 * FLAG BLOCK
 */
trait Flag {
    fn fmt_flags(&self) -> String;
}

impl Flag for Arg {
    fn fmt_flags(&self) -> String {
        let short = match self.get_short() {
            Some(value) => format!("-{}", value),
            None => String::new(),
        };
        let long = match self.get_long() {
            Some(value) => format!("--{}", value),
            None => String::new(),
        };
        let values = match self.get_action().takes_values() {
            true => match self.get_value_names() {
                // TODO: What if multiple names are provided?
                Some([]) => Vec::new(),
                Some(value) => value.iter().map(|f| format!("<{}>", f)).collect::<Vec::<String>>(),
                None => vec![format!("<{}>", self.get_id())],
            },
            false => Vec::new(),
        };

        // print short arg, and add a comma if a long arg exists
        let mut s = format!("{:min$}", short, min = 2);

        if long.len() > 0 {
            // Add a comma if there is a long arg, otherwise just a space
            if short.len() > 0 {
                s.push_str(", ");
            } else {
                s.push_str("  ");
            }
            s.push_str(format!("{}", long).as_str());
        };

        if values.len() > 0 {
            s.push_str(format!(" {}", values.join(" ")).as_str());
        }

        return s;
    }
}


fn build_cmd(command: &Command) -> &Command {
    let hc = &command;
    let fmt_command = hc.fmt_cmd(Vec::new());

    let mut children_commands: Vec<FmtCommands> = Vec::new();
    let parents: Vec<String> = Vec::new();
    extract_subcommands(command, &mut children_commands, parents);

    let mut handlebars = Handlebars::new();
    handlebars.register_template_file("template", "/Users/roloenusa/projects/personal/clap-show-rs/src/test.html").unwrap();
    // handlebars.register_template_file("test_p", "test_p.html").unwrap();


    println!("{}", handlebars.render("template", &Page{ main: fmt_command, subcommands: children_commands }).unwrap());

    hc
}

fn extract_subcommands(subcommand: &Command, children_commands: &mut Vec<FmtCommands>, parents: Vec<String>) {
    let mut parents: Vec<String> = parents;
    parents.push(subcommand.get_name().to_string());

    for subcommand in subcommand.get_subcommands() {
        let subcommand = subcommand.to_owned();
        children_commands.push(subcommand.fmt_cmd(parents.clone()));

        extract_subcommands(&subcommand, children_commands, parents.clone());
    }
}
