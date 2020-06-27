use experiment_differ::cli::Args;
use std::error::Error;
use structopt::StructOpt;

fn main() -> Result<(), Box<dyn Error>> {
    let _args = Args::from_args();
    Ok(())
}
