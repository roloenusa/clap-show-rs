//! Generate documentation for clap command-line tools


#[allow(dead_code)]
// Ensure that doc tests in the README.md file get run.
#[doc(hidden)]
mod test_readme {
    #![doc = include_str!("../README.md")]
}

use std::fmt::Write;

use clap::{Arg, Command};
use ramhorns::{Content, Template};

const OPTIONS_TEMPLATE: &str = "
<div class=\"code\">
  <h4>Options</h4>
  {options}
</div>";

const ARG_TEMPLATE: &str = "
<div class=\"code\">
    <h4>Options</h4>
    {{#options}}
    <div class=\"\">
        <div class=\"\"><pre>{{flags}}<pre></div><div><pre>{{description}}</pre></div>
    </div>
    {{/options}}
</div>";



#[derive(Content)]
struct FmtArg<'a> {
    flags: &'a str,
    description: &'a str
}


#[derive(Content)]
struct FmtOptions<'a> {
    options: Vec<FmtArg<'a>>,
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

struct HlpCmd {
    name: String,
    long_flag: String,
    short_flag: String,

    // long_about: String,
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

        // let long_about = match cmd.get_long_about() {
        //     Some(value) => value.to_string(),
        //     None => String::new(),
        // };

        let long_flag = match cmd.get_long_flag() {
            Some(value) => format!("--{}", value),
            None => String::new(),
        };

        let short_flag = match cmd.get_short_flag() {
            Some(value) => format!("-{}", value),
            None => String::new(),
        };

        HlpCmd {
            name: cmd.get_name().to_string(),
            long_flag,
            short_flag,
            // long_about,
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

            let mut options: Vec<FmtArg> = Vec::new();
            for arg in self.options.iter() {

                let flags = arg.fmt_flags();
                let flags = flags.to_owned();
                options.push(FmtArg {
                    flags: "pew",
                    description: &arg.description,
                });
            }

            let tpl = Template::new(ARG_TEMPLATE).unwrap();
            let rendered = tpl.render(&FmtOptions {
                options
            });

            println!("{}", rendered);
        }
    }

    fn calculate_width(&self) -> usize {
        let mut width: usize = 0;
        for cmd in self.commands.iter() {
            let name_length = match cmd.name.len() {
                0 => 0,
                value => value + 2, // value + comma + space
            };

            let short_length = match cmd.short_flag.len() {
                0 => 0,
                value => value + 2, // value + single-dash + comma + space
            };

            let long_length = match cmd.long_flag.len() {
                0 => 0,
                value => value + 1, // value + double-dash + comma
            };

            width = std::cmp::max(width, name_length + short_length + long_length);
        }
        width += 2; // 2 trailing spaces

        return width;
    }

    fn print_commands(&self) {
        if self.commands.is_empty() {
            return;
        }

        let width = self.calculate_width();

        println!("Commands");

        let mut f = String::new();
        for cmd in self.commands.iter() {
            let name = match &cmd.name.is_empty() {
                false => format!("{}, ", cmd.name),
                true => "".to_string(),
            };

            let short_flag = match &cmd.short_flag.is_empty() {
                false => format!("{}, ", cmd.short_flag),
                true => "".to_string(),
            };

            let long_flag = match &cmd.long_flag.is_empty() {
                false => format!("{}", cmd.long_flag),
                true => "".to_string(),
            };

            writeln!(
                f,
                "{:width$} {}",
                format!("  {}{}{}", name, short_flag, long_flag),
                &cmd.short_about
            )
            .unwrap();
        }
        println!("{}", f);
        println!("\n");
    }
}

#[derive(Clone)]
struct HlpArg {
    short: String,
    long: String,
    values: Vec<String>,
    description: String,

}

impl HlpArg {
    fn new(arg: &Arg) -> Self {
        let short = match arg.get_short() {
            Some(value) => format!("-{}", value),
            None => String::new(),
        };
        let long = match arg.get_long() {
            Some(value) => format!("--{}", value),
            None => String::new(),
        };
        let values = match arg.get_action().takes_values() {
            true => match arg.get_value_names() {
                // TODO: What if multiple names are provided?
                Some([]) => Vec::new(),
                Some(value) => value.iter().map(|f| format!("&lt;{}&gt;", f)).collect::<Vec::<String>>(),
                None => vec![format!("&lt;{}&gt;", arg.get_id())],
            },
            false => Vec::new(),
        };

        let description = match arg.get_help() {
            Some(value) => value.to_string(),
            None => String::new(),
        };

        HlpArg {
            short,
            long,
            values,
            description,
        }
    }

    // fn print(&self, length: usize) {
    //     let mut s = String::new();
    //     self.fmt_flags(&mut s).unwrap();

    //     write!(s, "  {}", self.description).expect("error");
    //     println!("{}", s);
    // }

    // fn fmt(&self, s: &mut String, length: usize) -> std::fmt::Result {
    //     // 2 spaces to indent
    //     write!(s, "  ")?;

    //     let mut long_comma = String::from("  ");
    //     if self.long.len() > 0 {
    //         long_comma = String::from(", ")
    //     };

    //     // print short arg, and add a comma if a long arg exists
    //     write!(
    //         s,
    //         "{:width$}",
    //         format!("{}{}", self.short, long_comma),
    //         width = 2
    //     )?;

    //     if self.long.len() > 0 {
    //         // Add a comma if a short flag was there.
    //         if self.short.len() > 0 {
    //             write!(s, ",")?
    //         }
    //         write!(s, "{}", self.short)?;
    //     };

    //     let width = length + 2;
    //     write!(s, "{}", format!("{:width$}", self.long))?;

    //     write!(s, "<{}>  ", self.value)?;

    //     println!("{}", s);

    //     return Ok(());
    // }

    // fn fmt_flags(&self, s: &mut String) -> std::fmt::Result {
    //     // print short arg, and add a comma if a long arg exists
    //     write!(s, "{:min$}", self.short, min = 2)?;

    //     if self.long.len() > 0 {
    //         // Add a comma if there is a long arg, otherwise just a space
    //         if self.short.len() > 0 {
    //             write!(s, ", ")?;
    //         } else {
    //             write!(s, "  ")?;
    //         }
    //         write!(s, "{}", self.long)?;
    //     };

    //     if self.values.len() > 0 {
    //         write!(s, " {}", self.values.join(" "))?;
    //     }

    //     return Ok(());
    // }

    fn fmt_flags(&self) -> String {
        // print short arg, and add a comma if a long arg exists
        let mut s = format!("{:min$}", self.short, min = 2);

        if self.long.len() > 0 {
            // Add a comma if there is a long arg, otherwise just a space
            if self.short.len() > 0 {
                s.push_str(", ");
            } else {
                s.push_str("  ");
            }
            s.push_str(format!("{}", self.long).as_str());
        };

        if self.values.len() > 0 {
            s.push_str(format!(" {}", self.values.join(" ")).as_str());
        }

        return s;
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
