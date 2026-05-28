mod app;
mod cli;
mod mcp;
mod output;

fn main() -> std::process::ExitCode {
    app::run()
}
