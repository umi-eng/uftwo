//! uftwo CLI tool.

mod convert;

use clap::Parser;

#[derive(Parser)]
#[clap(name = "uftwo", about = "UF2 utility")]
struct Cli {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    /// Convert a binary or file to a UF2 file.
    Convert(convert::Cmd),
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.subcommand {
        Subcommand::Convert(cmd) => cmd.run(),
    }
}
