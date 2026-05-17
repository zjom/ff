use clap::Parser;
use ff::cli::Cli;
use ff::eval::eval_file;
use ff::repl::Repl;

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
