use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::mouse::Cursor;
use sdl3::mouse::MouseButton;
use sdl3::mouse::MouseState;
use sdl3::mouse::MouseWheelDirection;
use sdl3::pixels::{Color, FColor};
use sdl3::rect::Point;
use sdl3::render::{Canvas, FPoint, Vertex};
use sdl3::sys::mouse::SDL_GetMouseState;
use sdl3::video::Window;
use std::any::Any;
use std::fmt::Debug;
use std::time::Duration;

// "Borrowed"
fn generate_circle_fan(
    center: FPoint,
    radius: f32,
    segments: usize,
    color: FColor,
) -> (Vec<Vertex>, Vec<i32>) {
    let mut vertices: Vec<Vertex> = vec![];
    let mut indices = vec![];
    // angle step
    let step = (std::f32::consts::PI * 2.0) / segments as f32;

    // center
    vertices.push(Vertex {
        position: FPoint::new(center.x, center.y),
        color,
        tex_coord: FPoint::new(0.0, 0.0),
    });

    for segment in 0..=segments {
        let angle = step * segment as f32;

        vertices.push(Vertex {
            position: FPoint::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            ),
            color,
            tex_coord: FPoint::new(0.0, 0.0),
        });

        // Don't want to include a "line" triangle with just 2 verts
        if segment != 0 {
            indices.push(0); // center
            indices.push(segment as i32);
            indices.push(segment as i32 + 1);
        }
    }

    (vertices, indices)
}

#[derive(Clone, Copy)]
struct Body {
    x: f64,
    y: f64,
    v_x: f64,
    v_y: f64,
    mass: f64,
    pinned: bool,
}

impl Body {
    pub fn new(x: f64, y: f64, v_x: f64, v_y: f64, mass: f64, pinned: bool) -> Self {
        Body {
            x,
            y,
            v_x,
            v_y,
            mass,
            pinned,
        }
    }

    pub fn render(&self, canvas: &mut Canvas<Window>, pan_x: f32, pan_y: f32, zoom: f32) {
        let color: FColor = {
            if self.mass > 0.0 {
                FColor::RED
            } else {
                FColor::BLUE
            }
        };

        let (vertices, indices) = generate_circle_fan(
            FPoint::new(
                (self.x as f32 * zoom) + pan_x,
                (self.y as f32 * zoom) + pan_y,
            ), // center
            self.mass.abs().sqrt() as f32 * zoom.max(0.1), // radius
            500,                                           // segments
            color,                                         // color
        );

        canvas.render_geometry(&vertices, None, &indices).unwrap();
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl3 demo: Video", 800, 600)
        .position_centered()
        .resizable()
        .fullscreen()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;

    let mut bodies: Vec<Body> = vec![];

    let mut panning = false;
    let mut pan_x = 0.0;
    let mut pan_y = 0.0;

    let mut zoom = 1.0;

    // for i in 0..3 {}

    bodies.push(Body::new(660.6, 300.6, 0.0, 0.0, 1000.0, true));
    bodies.push(Body::new(100.6, 300.6, 0.0, 0.0, -1000.0, true));

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::MouseButtonDown {
                    mouse_btn,
                    timestamp,
                    window_id,
                    which,
                    clicks,
                    x,
                    y,
                } => {
                    if mouse_btn == MouseButton::Middle {
                        panning = true;
                    }

                    if mouse_btn == MouseButton::Left || mouse_btn == MouseButton::Right {
                        let mass_mul = {
                            if mouse_btn == MouseButton::Right {
                                -1.0
                            } else {
                                1.0
                            }
                        };
                        bodies.push(Body::new(
                            ((x - pan_x) / zoom).into(),
                            ((y - pan_y) / zoom).into(),
                            0.0,
                            0.0,
                            100.0 * mass_mul,
                            false,
                        ));
                    }
                }

                Event::MouseMotion {
                    timestamp,
                    window_id,
                    which,
                    mousestate,
                    x,
                    y,
                    xrel,
                    yrel,
                } => {
                    if panning {
                        pan_x += xrel;
                        pan_y += yrel;
                    }
                }

                Event::MouseButtonUp { mouse_btn, .. } => {
                    if mouse_btn == MouseButton::Middle {
                        panning = false;
                    }
                }

                Event::MouseWheel {
                    timestamp,
                    window_id,
                    which,
                    x,
                    y,
                    direction,
                    mouse_x,
                    mouse_y,
                } => {
                    let delta_z = (y / 20.0) * zoom;
                    zoom += delta_z;

                    pan_x += pan_x * delta_z;
                    pan_y += pan_y * delta_z;

                    pan_x -= mouse_x * delta_z;
                    pan_y -= mouse_y * delta_z;
                }
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));

        for body in &bodies {
            body.render(&mut canvas, pan_x, pan_y, zoom);
        }

        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...

        for body2 in bodies.clone() {
            for mut body in &mut bodies {
                if body.pinned {
                    continue;
                }

                if body.x == body2.x && body.y == body2.y {
                    continue;
                }

                let dist_sq = (body.x - body2.x).powi(2) + (body.y - body2.y).powi(2);
                // f = m1m2/r^2

                let mut force = -body2.mass.abs() / dist_sq;

                if body.mass.signum() != body2.mass.signum() {
                    force *= -1.0;
                }

                let angle = f64::atan2(body2.y - body.y, body2.x - body.x);

                body.v_x += force * angle.cos();
                body.v_y += force * angle.sin();
            }
        }

        for mut body in &mut bodies {
            body.x += body.v_x;
            body.y += body.v_y;
        }
    }

    Ok(())
}
