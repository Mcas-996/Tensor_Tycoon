fn main() {
    if let Err(error) = monopoly_cli::ui::run() {
        eprintln!("monopoly_cli: {error}");
        std::process::exit(1);
    }
}
