mod cli;
mod gui;
mod player;

use cli::CliOptions;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: CliOptions = argh::from_env();

    if !args.no_gui {
        return Ok(gui::run()?);
    }

    if args.input.is_none() {
        eprintln!("No input was provided.");
        return Ok(());
    }

    Ok(())
}
