use clap::{
    ValueEnum,
    Parser,
    Subcommand,
};

#[derive(Parser)]
#[command(version, about = "A good music status display for polybar", long_about = None)]
pub struct Args {
    #[arg(short, long, default_value_t = 1500, help="The frequency, in milliseconds, that the display window will update")]
    pub update_frequency: u64,

    #[arg(short, long, default_value_t = 15, help="The length of the display window")]
    pub banner_size: usize,

    #[arg(short, long, help="Include player controls in the module")]
    pub include_controls: bool,

    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Operation {
    PREVIOUS,
    TOGGLE,
    NEXT,
}

#[derive(Subcommand)]
pub enum Command {
    #[command(about = "Control the player instead of running the display daemon")]
    Control {
        #[arg(short, long, help="The player to control")]
        player: String,

        #[arg(short, long, help="The operation to execute")]
        operation: Operation,
    },
}

