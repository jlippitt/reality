use clap::Parser;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use system::{Device, DeviceOptions, DisplayTarget};
use winit::dpi::Size;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowBuilder;

mod log;

#[derive(Parser, Debug)]
struct Args {
    rom_path: PathBuf,

    #[arg(short, long)]
    pif_data_path: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let rom_data = fs::read(args.rom_path)?;

    let pif_data = if let Some(pif_data_path) = args.pif_data_path {
        Some(fs::read(pif_data_path)?)
    } else {
        None
    };

    let _guard = log::init()?;

    let event_loop = EventLoop::new()?;

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Reality")
            .with_min_inner_size(Size::Physical((640, 480).into()))
            .build(&event_loop)?,
    );

    let mut device = Device::new(DeviceOptions {
        display_target: DisplayTarget {
            window: window.clone(),
            width: window.inner_size().width,
            height: window.inner_size().height,
        },
        pif_data,
        rom_data,
    })?;

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
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
                WindowEvent::Resized(size) => {
                    device.resize(size.width, size.height);
                }
                WindowEvent::RedrawRequested => {
                    device.render().unwrap();
                }
                _ => (),
            },
            Event::AboutToWait => {
                while !device.step() {}
                window.request_redraw();
            }
            _ => (),
        }
    })?;

    Ok(())
}
