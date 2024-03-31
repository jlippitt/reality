use clap::Parser;
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use system::Device;

mod log;

#[derive(Deserialize, Debug)]
struct Config {
    pif_data_path: String,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    config_path: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let config: Config = {
        let config_data = fs::read_to_string(args.config_path)?;
        toml::from_str(&config_data)?
    };

    let pif_data = {
        let pif_data_path = shellexpand::full(&config.pif_data_path)?;
        fs::read(pif_data_path.as_ref())?
    };

    let _guard = log::init()?;

    let mut device = Device::new(pif_data);

    loop {
        device.step();
    }
}
