use clap::Parser;
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use system::Device;
use winit::dpi::Size;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowBuilder;

mod log;

#[derive(Deserialize, Debug)]
struct Config {
    pif_data_path: String,
}

#[derive(Parser, Debug)]
struct Args {
    rom_path: PathBuf,

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

    let rom_data = fs::read(args.rom_path)?;

    let _guard = log::init()?;

    let event_loop = EventLoop::new()?;

    let window = WindowBuilder::new()
        .with_title("Reality")
        .with_min_inner_size(Size::Physical((640, 480).into()))
        .build(&event_loop)?;

    let mut device = Device::new(pif_data, rom_data);

    event_loop.run(move |event, elwt| match event {
        Event::WindowEvent {
            event: window_event,
            ..
        } => match window_event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => {
                elwt.exit();
            }
            WindowEvent::RedrawRequested => {
                println!("Redraw");
            }
            _ => (),
        },
        Event::AboutToWait => {
            while !device.step() {}
            window.request_redraw();
        }
        _ => (),
    })?;

    Ok(())
}
