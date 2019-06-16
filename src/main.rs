#[macro_use]
extern crate glium;
extern crate rand;

mod gol;
mod state;

use glium::texture::Texture2d;
use glium::uniforms::MagnifySamplerFilter::Nearest;
use glium::uniforms::SamplerWrapFunction::Repeat;
use glium::{glutin, Display, Surface};

use glutin::dpi::LogicalSize;
use glutin::VirtualKeyCode;

use gol::GoL;
use state::GameState;

#[derive(PartialEq)]
enum ProgramStatus {
    Done,
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

fn main() {
    loop {
        match run_shader() {
            Ok(status) => {
                if status == ProgramStatus::Done {
                    return;
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                return;
            }
        }
    }
}

fn init_display(events_loop: &glutin::EventsLoop) -> Result<Display, &'static str> {
    let window_size = LogicalSize::new(512.0, 512.0);
    let window = glutin::WindowBuilder::new().with_title("Game of Life");
    let context = glutin::ContextBuilder::new();

    Display::new(window.with_dimensions(window_size), context, &events_loop)
        .map_err(|_| "Could not initialize the display.")
}

fn fullscreen() -> Vec<Vertex> {
    vec![
        Vertex {
            position: [-1.0, -1.0],
            tex_coords: [0.0, 0.0],
        },
        Vertex {
            position: [-1.0, 1.0],
            tex_coords: [0.0, 1.0],
        },
        Vertex {
            position: [1.0, 1.0],
            tex_coords: [1.0, 1.0],
        },
        Vertex {
            position: [-1.0, -1.0],
            tex_coords: [0.0, 0.0],
        },
        Vertex {
            position: [1.0, 1.0],
            tex_coords: [1.0, 1.0],
        },
        Vertex {
            position: [1.0, -1.0],
            tex_coords: [1.0, 0.0],
        },
    ]
}

fn run_shader() -> Result<ProgramStatus, &'static str> {
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

    let mut game = GoL::new((128, 128));
    game.randomize();

    let mut state = GameState::new();
    let mut running = true;

    while state.open {
        state.tick();

        if running {
            game.step();
        }

        if state.key_pressed(VirtualKeyCode::Space) {
            running = !running;
        }

        if state.key_pressed(VirtualKeyCode::R) {
            game.randomize();
        }

        let texture = Texture2d::new(&display, game.as_raw_image_2d()).unwrap();
        let uniforms = uniform! {
            texture: texture.sampled().magnify_filter(Nearest).wrap_function(Repeat),
            time: state.time
        };

        let mut target = display.draw();

        target.clear_color(1.0, 1.0, 1.0, 1.0);
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
                state.update(event);
            }
        });
    }

    Ok(ProgramStatus::Done)
}
