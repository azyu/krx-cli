fn main() {
    if let Err(err) = krx_cli::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
