use clap::{Args, CommandFactory, Parser, Subcommand};

extern crate clap_show;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Positional {
    /// Positional required argument. Lets change this
    ///
    /// Long description of the required positional argument
    /// when i say long, this is at least a paragraph long and may even
    /// be considerably lonter.
    ///
    /// I have even thrown a second paragraph to test how this works over very
    /// long prompts.
    pos_required: String,

    /// Positional optional argument
    ///
    /// Long description of the optional positional argument
    pos_optional: Option<String>,

}

/// This explains how the application works on details. Probably a good to
/// have an introduction to the commands and the purpose of it.
#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    /// Example of how arguments are treated on the application documentation
    Arguments(ArgumentArgs),

    /// Positional short help
    ///
    /// This is the long help prompt via derived macros for positional arguments
    /// This long help should be respected. This long should be extended over
    /// several lines and should wrap nicely around a paragraph.
    ///
    /// A third paragraph should also be displayed on the help. With all the
    /// content properly shown and displayed around the stuff.
    Positional(Positional),

    /// flag short help
    ///
    /// flag arguments long help
    Flags(FlagArgs),

    /// markdown short help
    ///
    /// markdown arguments long help
    Markdown(MarkdownArgs),
}

#[derive(Args)]
struct ArgumentArgs {
    optional: Option<String>,

    multiple: Vec<String>,
}


#[derive(Args)]
struct FlagArgs {
    /// Flag short help
    ///
    /// Flag long help
    #[arg(short='f', long="flag")]
    flag: String,
}


#[derive(Args)]
struct MarkdownArgs {}

fn main() {
    let cli = Cli::parse();
    let command = Cli::command();

    match cli.command {
       _ => clap_show::help_command(&command)
    };
}

