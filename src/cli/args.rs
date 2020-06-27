use structopt::StructOpt;
use super::ReadableFilename;

//
// NOTE: The following doc comments on the `Args` struct are parsed by the `StructOpt`
// macros to define the `--help` output of the command-line interface...
//
/// Diff structured data files using key fields with high performance.
///
/// TODO: Add additional intro paragraph for the --help output here!
///
#[derive(Debug, StructOpt)]
#[structopt(name = "trecs")]
pub struct Args {
    /// Specify an alternate trecs file
    #[structopt(
        short,
        long,
        parse(try_from_os_str = ReadableFilename::try_from_os_str),
        name = "FILE",
        default_value = "trecs.yml"
    )]
    file: ReadableFilename,
}



