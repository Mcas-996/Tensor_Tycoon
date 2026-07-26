mod cli;

use cli::StartupAction;

fn main() {
    match cli::parse(std::env::args_os().skip(1)) {
        Ok(StartupAction::Help) => println!("{}", cli::HELP),
        Ok(StartupAction::Version) => println!("{}", cli::VERSION),
        Ok(StartupAction::Run) => {
            if let Err(error) = tensor_tycoon::ui::run() {
                eprintln!("tensor_tycoon: {error}");
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("tensor_tycoon: error: {error}");
            eprintln!("Try 'tensor_tycoon --help' for more information.");
            std::process::exit(2);
        }
    }
}
