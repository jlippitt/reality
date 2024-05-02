use gilrs::{Axis, Button, Error, Event, EventType, Gilrs};
use std::time::SystemTime;
use system::JoypadState;

pub struct Gamepad {
    gilrs: Gilrs,
    joypad_state: [JoypadState; 4],
}

impl Gamepad {
    pub fn new() -> Result<Self, Error> {
        let mut gilrs = Gilrs::new()?;

        if let Some((id, gamepad)) = gilrs.gamepads().next() {
            let left_z = gamepad.axis_code(Axis::LeftZ).unwrap();
            let right_z = gamepad.axis_code(Axis::RightZ).unwrap();

            // Force Z axes into 'off' position
            gilrs.update(&Event {
                id,
                event: EventType::AxisChanged(Axis::LeftZ, -1.0, left_z),
                time: SystemTime::now(),
            });

            gilrs.update(&Event {
                id,
                event: EventType::AxisChanged(Axis::RightZ, -1.0, right_z),
                time: SystemTime::now(),
            });
        }

        Ok(Self {
            gilrs,
            joypad_state: Default::default(),
        })
    }

    pub fn handle_events(&mut self) -> &[JoypadState; 4] {
        let mut should_update = false;

        while let Some(event) = self.gilrs.next_event() {
            match event.event {
                EventType::ButtonPressed(..)
                | EventType::ButtonReleased(..)
                | EventType::AxisChanged(..) => should_update = true,
                _ => (),
            }
        }

        if should_update {
            self.update();
        }

        &self.joypad_state
    }

    fn update(&mut self) {
        let Some((_, gamepad)) = self.gilrs.gamepads().next() else {
            return;
        };

        self.joypad_state[0] = JoypadState {
            a: gamepad.is_pressed(Button::South),
            // North and west appear swapped for me, though this may just be an Xbox controller issue?
            b: gamepad.is_pressed(Button::North),
            c_up: gamepad.value(Axis::RightStickY) >= 0.75,
            c_down: gamepad.is_pressed(Button::East) || gamepad.value(Axis::RightStickY) <= -0.75,
            c_left: gamepad.is_pressed(Button::West) || gamepad.value(Axis::RightStickX) <= -0.75,
            c_right: gamepad.value(Axis::RightStickX) >= 0.75,
            l: gamepad.is_pressed(Button::LeftTrigger2) || gamepad.value(Axis::LeftZ) >= -0.75,
            r: gamepad.is_pressed(Button::RightTrigger)
                || gamepad.is_pressed(Button::RightTrigger2)
                || gamepad.value(Axis::RightZ) >= -0.75,
            z: gamepad.is_pressed(Button::LeftTrigger),
            start: gamepad.is_pressed(Button::Start),
            dpad_up: gamepad.is_pressed(Button::DPadUp),
            dpad_down: gamepad.is_pressed(Button::DPadDown),
            dpad_left: gamepad.is_pressed(Button::DPadLeft),
            dpad_right: gamepad.is_pressed(Button::DPadRight),
            axis_x: (gamepad.value(Axis::LeftStickX)
                * (83.0 - (gamepad.value(Axis::LeftStickY).abs() * 17.0)))
                as i8,
            axis_y: (gamepad.value(Axis::LeftStickY)
                * (83.0 - (gamepad.value(Axis::LeftStickX).abs() * 17.0)))
                as i8,
        };
    }
}
