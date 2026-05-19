use clap::Parser;
use f2::cli::Cli;
use f2::eval::eval_file;
use f2::repl::Repl;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(path) = cli.file {
        eval_file(path)?;
    } else {
        let mut repl: Repl = cli.repl.try_into()?;
        repl.run()?;
    }

    Ok(())
}
