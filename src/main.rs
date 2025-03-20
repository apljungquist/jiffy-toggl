use clap::Parser;
use jiffy2toggl::Cli;


fn main() {
    env_logger::init();
    Cli::parse().exec().unwrap();
}
