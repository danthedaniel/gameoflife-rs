use std::collections::HashMap;
use std::sync::mpsc::{channel, sync_channel, Receiver, SyncSender};
use std::thread;
use std::thread::JoinHandle;
use std::time::SystemTime;

use glium::glutin::{ElementState, MouseButton, VirtualKeyCode, WindowEvent};
use glium::texture::RawImage2d;

use super::gol::GoL;

pub const GAME_WIDTH: usize = 64;
pub const GAME_HEIGHT: usize = 64;

struct ButtonState {
    /// Whether the button/key is pressed.
    pub down: bool,
    /// Whether or not the current element state has been seen.
    pub seen: bool,
}

impl ButtonState {
    pub fn new(state: ElementState, current_down: bool) -> Self {
        let down = match state {
            ElementState::Pressed => true,
            ElementState::Released => false,
        };

        Self {
            down: down,
            seen: current_down == down,
        }
    }
}

const TICK_RATE: usize = 1; // Hz
const TICK_DELAY: f32 = 1.0 / TICK_RATE as f32;

/// Events sent to the GoL simulation thread.
pub enum Tick {
    /// Simulate the next step.
    Continue,
    /// Reset and randomize the simulation.
    Randomize,
    /// Terminate the simulation
    Quit,
}

pub struct GameState {
    /// Mapping from keys to whether or not they're pressed.
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
    /// When the last tick occurred.
    last_tick: f32,
    /// Number of ticks simulated so far.
    pub tick_count: usize,
    /// Whether the simulation is running.
    pub running: bool,
    /// GoL simulation thread,
    simulation_thread: Option<JoinHandle<()>>,
    /// Sender to provide events to the simulation thread from the main thread.
    tick_sender: SyncSender<Tick>,
    /// Receiver that receives Game of Life textures from the simulation thread.
    pub tex_receiver: Receiver<RawImage2d<'static, u8>>,
}

impl GameState {
    pub fn new() -> Self {
        let (simulation_thread, tick_sender, tex_receiver) = GameState::start_thread();

        // Run at least one tick so that there's a texture to receive immediately
        // after new() is called.
        tick_sender.send(Tick::Continue).unwrap();

        Self {
            keyboard: HashMap::new(),
            mouse: HashMap::new(),
            cursor: (0.0, 0.0),
            open: true,
            start: SystemTime::now(),
            time: 0.0,
            last_tick: 0.0,
            tick_count: 0,
            running: true,
            simulation_thread: Some(simulation_thread),
            tick_sender: tick_sender,
            tex_receiver: tex_receiver,
        }
    }

    fn start_thread() -> (
        JoinHandle<()>,
        SyncSender<Tick>,
        Receiver<RawImage2d<'static, u8>>,
    ) {
        let (tick_sender, tick_receiver) = sync_channel::<Tick>(2);
        let (tex_sender, tex_receiver) = channel::<RawImage2d<u8>>();

        let simulation_thread = thread::spawn(move || {
            let mut game = GoL::new((GAME_WIDTH, GAME_HEIGHT));
            game.randomize();

            loop {
                use Tick::*;

                match tick_receiver.recv() {
                    Ok(Continue) => {
                        game.step();
                        let texture = game.as_raw_image_2d();
                        tex_sender.send(texture).unwrap();
                    }
                    Ok(Randomize) => {
                        game.randomize();
                        let texture = game.as_raw_image_2d();
                        tex_sender.send(texture).unwrap();
                    }
                    Ok(Quit) => {
                        return;
                    }
                    Err(e) => {
                        eprintln!("GoL Thread tick_receiver.recv() error: {:?}", e);
                        return;
                    }
                }
            }
        });

        (simulation_thread, tick_sender, tex_receiver)
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

    pub fn send(&self, tick: Tick) {
        self.tick_sender.send(tick).unwrap();
    }

    /// Updates to state ran per-frame.
    pub fn frame(&mut self) {
        let time_millis = SystemTime::now()
            .duration_since(self.start)
            .unwrap()
            .as_millis();
        self.time = time_millis as f32 / 1000.0;

        // Run next simulation frame if enough time has passed
        if self.running || self.key_down(VirtualKeyCode::C) {
            while self.last_tick < self.time {
                self.last_tick += TICK_DELAY;
                self.tick();
            }
        }
    }

    /// Updates to state ran per-tick.
    pub fn tick(&mut self) {
        match self.tick_sender.try_send(Tick::Continue) {
            Ok(()) => {
                self.tick_count += 1;
            }
            Err(_) => {
                eprintln!("Failed to send tick to simulation_thread");
            }
        }
    }

    /// The number of seconds of game execution (excludes time when paused).
    pub fn simulation_time(&self) -> f32 {
        let delta = self.time - self.last_tick;
        let interpolation = if !self.running {
            0f32
        } else if delta < TICK_DELAY {
            delta
        } else {
            TICK_DELAY
        };
        return self.tick_count as f32 * TICK_DELAY + interpolation;
    }

    /// Apply an event's changes to state.
    pub fn consume_event(&mut self, event: WindowEvent) {
        use WindowEvent::*;

        match event {
            CloseRequested => {
                self.open = false;
            }
            KeyboardInput { input, .. } => {
                if let Some(keycode) = input.virtual_keycode {
                    let current_down = self.key_down(keycode);
                    self.keyboard
                        .insert(keycode, ButtonState::new(input.state, current_down));
                }
            }
            MouseInput { state, button, .. } => {
                let current_down = self.mouse_down(button);
                self.mouse
                    .insert(button, ButtonState::new(state, current_down));
            }
            CursorMoved { position, .. } => {
                self.cursor = (position.x, position.y);
            }
            _ => {}
        }
    }
}

impl Drop for GameState {
    fn drop(&mut self) {
        self.send(Tick::Quit);
        self.simulation_thread
            .take()
            .expect("simulation_thread is None")
            .join()
            .expect("simulation_thread can't be joined");

        println!(
            "Actual tick rate: {:.2}",
            self.tick_count as f32 / self.time
        );
    }
}
