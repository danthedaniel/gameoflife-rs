use std::collections::HashMap;
use std::time::SystemTime;

use glium::glutin::{ElementState, MouseButton, VirtualKeyCode, WindowEvent};

struct ButtonState {
    /// Whether the button/key is pressed.
    pub down: bool,
    /// Whether or not the current element state has been seen.
    pub seen: bool,
}

impl ButtonState {
    pub fn new(down: bool, seen: bool) -> Self {
        Self {
            down: down,
            seen: seen,
        }
    }
}

impl From<ElementState> for ButtonState {
    fn from(state: ElementState) -> Self {
        ButtonState::new(state_as_bool(state), false)
    }
}

pub struct GameState {
    /// Mapping from keys to whether or not it's pressed.
    keyboard: HashMap<VirtualKeyCode, ButtonState>,
    /// Mapping from buttons to whether or not they're pressed.
    mouse: HashMap<MouseButton, ButtonState>,
    /// Cursor location in pixels.
    pub cursor: (f64, f64),
    /// If the window is open.
    pub open: bool,
    /// Time at creation of state struct.
    start: SystemTime,
    /// How long the game has been running in seconds.
    pub time: f32,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            keyboard: HashMap::new(),
            mouse: HashMap::new(),
            cursor: (0.0, 0.0),
            open: true,
            start: SystemTime::now(),
            time: 0.0,
        }
    }

    /// Whether a key on the keyboard is currently pressed.
    pub fn key_down(&self, key: VirtualKeyCode) -> bool {
        self.keyboard
            .get(&key)
            .map(|ButtonState { down, .. }| *down)
            .unwrap_or(false)
    }

    /// Whether a key on the keyboard is newly pressed.
    pub fn key_pressed(&mut self, key: VirtualKeyCode) -> bool {
        if let Some(state) = self.keyboard.get_mut(&key) {
            if !state.seen {
                state.seen = true;
                return state.down;
            }
        }

        false
    }

    /// Whether a button on the mouse is currently pressed.
    pub fn mouse_down(&self, button: MouseButton) -> bool {
        self.mouse
            .get(&button)
            .map(|ButtonState { down, .. }| *down)
            .unwrap_or(false)
    }

    /// Whether a button on the mouse is newly pressed.
    pub fn mouse_pressed(&mut self, button: MouseButton) -> bool {
        if let Some(state) = self.mouse.get_mut(&button) {
            if !state.seen {
                state.seen = true;
                return state.down;
            }
        }

        false
    }

    /// Updates to game state ran per-frame.
    pub fn tick(&mut self) {
        let time_millis = SystemTime::now()
            .duration_since(self.start)
            .unwrap()
            .as_millis();
        self.time = time_millis as f32 / 1000.0;
    }

    /// Apply an event's changes to state.
    pub fn update(&mut self, event: WindowEvent) {
        use WindowEvent::*;

        match event {
            CloseRequested => {
                self.open = false;
            }
            KeyboardInput { input, .. } => {
                if let Some(keycode) = input.virtual_keycode {
                    self.keyboard
                        .insert(keycode, ButtonState::from(input.state));
                }
            }
            MouseInput { state, button, .. } => {
                self.mouse.insert(button, ButtonState::from(state));
            }
            CursorMoved { position, .. } => {
                self.cursor = (position.x, position.y);
            }
            _ => {}
        }
    }
}

#[inline]
fn state_as_bool(state: ElementState) -> bool {
    match state {
        ElementState::Pressed => true,
        ElementState::Released => false,
    }
}
