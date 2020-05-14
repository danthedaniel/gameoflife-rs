#[macro_use]
extern crate glium;
extern crate rand;

mod gol;
mod state;
mod vertex;

use std::collections::VecDeque;
use std::time::SystemTime;

use glium::texture::Texture2d;
use glium::uniforms::MagnifySamplerFilter::Nearest;
use glium::uniforms::SamplerWrapFunction::Repeat;
use glium::{glutin, Display, Surface};

use glutin::dpi::LogicalSize;
use glutin::VirtualKeyCode;

use state::{GameState, Tick, GAME_HEIGHT, GAME_WIDTH};
use vertex::fullscreen;

const WINDOW_WIDTH: f64 = 750.0;
const WINDOW_HEIGHT: f64 = 750.0;

#[derive(PartialEq)]
enum ProgramStatus {
    Done,
}

fn main() {
    let mut state = GameState::new();

    if let Err(msg) = run(&mut state) {
        eprintln!("{}", msg);
    }
}

fn init_display(events_loop: &glutin::EventsLoop) -> Result<Display, &'static str> {
    let window_size = LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT);
    let window = glutin::WindowBuilder::new().with_title("Game of Life");
    let context = glutin::ContextBuilder::new();

    Display::new(window.with_dimensions(window_size), context, &events_loop)
        .map_err(|_| "Could not initialize the display.")
}

fn run(state: &mut GameState) -> Result<ProgramStatus, &'static str> {
    // Set up window
    let mut events_loop = glutin::EventsLoop::new();
    let display = init_display(&events_loop)?;
    let shape = fullscreen();

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    // Compile shaders
    let vertex_shader_src = include_str!("shaders/main.vert");
    let fragment_shader_src = include_str!("shaders/main.frag");
    let program =
        glium::Program::from_source(&display, &vertex_shader_src, &fragment_shader_src, None)
            .map_err(|e| {
                eprintln!("{}", e);
                "GLSL compiler error"
            })?;

    // Collect an initial texture from the game thread.
    let mut texture = Texture2d::new(&display, state.tex_receiver.recv().unwrap()).unwrap();

    let frame_times_max_size: usize = 10;
    let mut frame_times: VecDeque<SystemTime> = VecDeque::new();

    let mut fullscreen = false;

    while state.open {
        frame_times.push_back(SystemTime::now());
        state.frame();

        if frame_times.len() > frame_times_max_size {
            let start_time = frame_times.pop_front().unwrap();
            let delay = SystemTime::now()
                .duration_since(start_time)
                .unwrap()
                .as_millis();
            println!(
                "{:.1} FPS\t{:.2} s\t{} ticks",
                1000.0 / (delay as f32 / frame_times_max_size as f32),
                state.simulation_time(),
                state.tick_count
            );
        }

        // Handle input events
        if state.key_pressed(VirtualKeyCode::Q) {
            state.open = false;
        }

        if state.key_pressed(VirtualKeyCode::Space) {
            state.running = !state.running;
        }

        if state.key_pressed(VirtualKeyCode::R) {
            state.send(Tick::Randomize);
        }

        if state.key_pressed(VirtualKeyCode::S) {
            state.tick();
        }

        if state.key_pressed(VirtualKeyCode::G) {
            state.send(Tick::RandomGlider);
        }

        if state.key_pressed(VirtualKeyCode::Return) {
            fullscreen = !fullscreen;

            if fullscreen {
                let wb = glutin::WindowBuilder::new()
                    .with_decorations(false)
                    .with_fullscreen(Some(events_loop.get_primary_monitor()));
                let cb = glutin::ContextBuilder::new();
                display.rebuild(wb, cb, &events_loop).unwrap();
            } else {
                let wb = glutin::WindowBuilder::new();
                let cb = glutin::ContextBuilder::new();
                display.rebuild(wb, cb, &events_loop).unwrap();
            }
        }

        // Update texture/uniforms
        if let Ok(new_texture) = state.tex_receiver.try_recv() {
            texture = Texture2d::new(&display, new_texture).unwrap();
        };

        let mut target = display.draw();
        let dimensions = target.get_dimensions();

        let uniforms = uniform! {
            texture: texture.sampled().magnify_filter(Nearest).wrap_function(Repeat),
            time: state.simulation_time(),
            board_size: [GAME_WIDTH as f32, GAME_HEIGHT as f32],
            window_size: [dimensions.0 as f32, dimensions.1 as f32],
        };

        target
            .draw(
                &vertex_buffer,
                &indices,
                &program,
                &uniforms,
                &Default::default(),
            )
            .map_err(|_| "Could not draw shader.")?;
        target.finish().unwrap();

        events_loop.poll_events(|event| {
            if let glutin::Event::WindowEvent { event, .. } = event {
                state.consume_event(event);
            }
        });
    }

    Ok(ProgramStatus::Done)
}
