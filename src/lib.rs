//! Generate documentation for clap command-line tools

static TEMPLATE_FILE: &'static str = include_str!("../data/template.html");
static CODE_PARTIAL: &'static str = include_str!("../data/usage-partial.html");

use clap::{Arg, Command};
use handlebars::Handlebars;
use serde_derive::Serialize;

#[derive(Serialize, Clone, Debug)]
struct FmtArg {
    flags: String,
    description: String,
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
    subcommands: Vec<FmtCommands>,
}

/// Format the help information for `command` as Markdown.
///
/// Output is printed to the standard output, using [`println!`].
pub fn write_help_factory<C: clap::CommandFactory>() {
    let command = C::command();

    help_command(&command);
}

/// Format the help information for `command` as Markdown.
///
/// Output is printed to the standard output, using [`println!`].
pub fn help_command(command: &clap::Command) {
    build_cmd(command);
}

fn get_usage(command: &mut Command) -> String {
    let parts = command.render_usage().to_string();
    let parts = parts.split(" ").collect::<Vec<&str>>();
    let slice = parts[2..]
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>();
    slice.join(" ").to_string()
}

fn fmt_cmd(command: &Command, parents: Vec<String>) -> FmtCommands {
    let description = match command.get_long_about() {
        Some(value) => value.to_string(),
        None => match command.get_about() {
            Some(value) => value.to_string(),
            None => String::new()
        },
    };

    let mut arguments: Vec<FmtArg> = Vec::new();
    let mut options: Vec<FmtArg> = Vec::new();
    for arg in command.get_arguments() {
        // Ignore the arguments that are hidden
        if arg.is_hide_set() {
            continue;
        }

        let fmt_arg = FmtArg {
            flags: fmt_flags(&arg),
            description: match arg.get_help_heading() {
                Some(value) => value.to_string(),
                None => match arg.get_long_help() {
                    Some(value) => value.to_string(),
                    None => String::new(),
                },
            },
        };

        if arg.is_positional() {
            arguments.push(fmt_arg);
        } else {
            options.push(fmt_arg);
        }
    }

    // Format the subcommands
    let mut subcommands: Vec<FmtCmd> = Vec::new();
    for command in command.get_subcommands() {
        subcommands.push(FmtCmd {
            name: command.get_name().to_string(),
            description: match command.get_about() {
                Some(t) => t.to_string(),
                _ => String::new(),
            },
        });
    }

    let mut cmd = command.clone();
    let usage = get_usage(&mut cmd);

    let mut ancestors = parents.clone();
    ancestors.push(command.get_name().to_string());

    FmtCommands {
        title: command.get_name().to_string(),
        usage,
        cmd_chain: ancestors.join(" "),
        description,
        commands: subcommands,
        arguments,
        options,
    }
}

/**
 * FLAG BLOCK
 */

fn fmt_flags(arg: &Arg) -> String {
    let short = match arg.get_short() {
        Some(value) => format!("-{}", value),
        None => String::new(),
    };
    let long = match arg.get_long() {
        Some(value) => format!("--{}", value),
        None => String::new(),
    };
    let mut values = match arg.get_action().takes_values() {
        true => match arg.get_value_names() {
            // TODO: What if multiple names are provided?
            Some([]) => Vec::new(),
            Some(value) => value
                .iter()
                .map(|f| {
                    match arg.is_required_set() {
                        true => format!("<{}>", f),
                        false  => format!("[{}]", f)
                    }
                })
                .collect::<Vec<String>>(),
            None => vec![format!("<{}>", arg.get_id())],
        },
        false => Vec::new(),
    };

    // Check if the argument takes multiple values
    let num_vals = arg.get_num_args().unwrap_or_else(|| 1.into());
    if num_vals.max_values() > 1 {
        values.push("...".to_string());
    }

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

fn build_cmd(command: &Command) -> &Command {
    let hc = &command;
    let fmt_command = fmt_cmd(&hc, Vec::new());

    let mut children_commands: Vec<FmtCommands> = Vec::new();
    let parents: Vec<String> = Vec::new();
    extract_subcommands(command, &mut children_commands, parents);

    let mut handlebars = Handlebars::new();

    handlebars.register_helper("paragraph", Box::new(paragraph));
    handlebars.register_helper("anchor", Box::new(anchor));

    handlebars
        .register_template_string("template", TEMPLATE_FILE)
        .expect("Unable to load base template");
    handlebars
        .register_template_string("usage-partial", CODE_PARTIAL)
        .expect("Unable to load base template");

    println!(
        "{}",
        handlebars
            .render(
                "template",
                &Page {
                    main: fmt_command,
                    subcommands: children_commands
                }
            )
            .unwrap()
    );

    hc
}

fn extract_subcommands(
    subcommand: &Command,
    children_commands: &mut Vec<FmtCommands>,
    parents: Vec<String>,
) {
    let mut parents: Vec<String> = parents;
    parents.push(subcommand.get_name().to_string());

    for subcommand in subcommand.get_subcommands() {
        let subcommand = subcommand.to_owned();
        children_commands.push(fmt_cmd(&subcommand, parents.clone()));

        extract_subcommands(&subcommand, children_commands, parents.clone());
    }
}

/// Implement a custom handlebar function that replaces "\n" for <br /> tags
/// This allows for proper paragraph inside the HTML so short and long descriptions
/// can be respected.
fn paragraph (h: &handlebars::Helper, _: &Handlebars, _: &handlebars::Context, _rc: &mut handlebars::RenderContext, out: &mut dyn handlebars::Output) -> handlebars::HelperResult {
    let param = h.param(0).unwrap();
    let param = match param.value().as_str() {
        Some(value) => value,
        None => "",
    };
    let param = param.replace("\n", "<br />");

    out.write(param.as_str())?;
    Ok(())
}

/// Implement a custom handlebar function that replaces spaces for dashes
/// This allows for better styled anchors
fn anchor(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _rc: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output
) -> handlebars::HelperResult {
    let param = h.param(0).unwrap();
    let param = match param.value().as_str() {
        Some(value) => value,
        None => "",
    };
    let param = param.replace(" ", "-");

    out.write(param.as_str())?;
    Ok(())
}

