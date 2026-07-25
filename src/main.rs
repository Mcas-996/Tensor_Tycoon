fn main() {
    if let Err(error) = tensor_tycoon::ui::run() {
        eprintln!("tensor_tycoon: {error}");
        std::process::exit(1);
    }
}
