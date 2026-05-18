use clap::Parser;

#[derive(Parser)]
struct X1Brief{
    #[arg(short = 's', long = "sum")]
    sum: bool,
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser)]
enum Commands {
    #[clap(name = "sum")]
    Summerize,
}
