mod cli;
mod phoenix;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::run()
}
