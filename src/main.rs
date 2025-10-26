use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseButton;
use sdl3::pixels::{Color, FColor};
use sdl3::rect::Point;
use sdl3::render::{Canvas, FPoint, Vertex};

use rayon::prelude::*;
use sdl3::video::Window;

// "Borrowed"
fn generate_circle_fan(
    center: FPoint,
    radius: f32,
    segments: usize,
    color: FColor,
) -> (Vec<Vertex>, Vec<i32>) {
    let mut vertices: Vec<Vertex> = vec![];
    let mut indices = vec![];

    if segments == 3 {
        // tri
        vertices.push(Vertex {
            position: FPoint::new(center.x, center.y - radius),
            color,
            tex_coord: FPoint::new(0.0, 0.0),
        });
        vertices.push(Vertex {
            position: FPoint::new(center.x - radius, center.y + radius),
            color,
            tex_coord: FPoint::new(0.0, 0.0),
        });
        vertices.push(Vertex {
            position: FPoint::new(center.x + radius, center.y + radius),
            color,
            tex_coord: FPoint::new(0.0, 0.0),
        });

        indices.push(0); // center
        indices.push(1);
        indices.push(2);

        return (vertices, indices);
    }

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

fn generate_circle_fan_color_edge(
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
            color: FColor::BLACK,
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
    init_x: f64,
    init_y: f64,
    v_x: f64,
    v_y: f64,
    mass: f64,
    pinned: bool,
    color: FColor,
}

impl Body {
    pub fn new(x: f64, y: f64, v_x: f64, v_y: f64, mass: f64, pinned: bool, color: FColor) -> Self {
        Body {
            x,
            y,
            init_x: x,
            init_y: y,
            v_x,
            v_y,
            mass,
            pinned,
            color,
        }
    }

    pub fn get_render(&self, pan_x: f32, pan_y: f32, zoom: f32) -> (Vec<Vertex>, Vec<i32>) {
        if self.color == FColor::BLACK {
            return (vec![], vec![]);
        }

        let (vertices, indices) = generate_circle_fan(
            FPoint::new(
                (self.x as f32 * zoom) + pan_x,
                (self.y as f32 * zoom) + pan_y,
            ), // center
            self.mass.abs().sqrt() as f32 * zoom.max(0.1), // radius
            3,                                             // segments
            self.color,                                    // color
        );

        return (vertices, indices);
    }

    fn render(&self, pan_x: f32, pan_y: f32, zoom: f32, canvas: &mut Canvas<Window>) {
        let (vertices, indices) = generate_circle_fan_color_edge(
            FPoint::new(
                (self.x as f32 * zoom) + pan_x,
                (self.y as f32 * zoom) + pan_y,
            ), // center
            self.mass.abs().sqrt() as f32 * zoom.max(0.1), // radius
            30,                                            // segments
            self.color,                                    // color
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
    let mut significant_bodies: Vec<Body> = vec![];

    let mut panning = false;
    let mut drawing = false;
    let mut pan_x = 0.0;
    let mut pan_y = 0.0;

    let mut zoom = 1.0;

    let mut render_mode = 0;

    let res = 0.1;
    let size = (600 as f64 * res) as i32;
    for x in -size..size {
        for y in -size / 2..size / 2 {
            bodies.push(Body::new(
                (x as f64 / res).into(),
                (y as f64 / res).into(),
                0.0,
                0.0,
                100.0,
                false,
                FColor::WHITE,
            ));
        }
    }

    significant_bodies.push(Body::new(
        0.0,
        -50.0,
        0.0,
        0.0,
        1000.0,
        true,
        FColor::YELLOW,
    ));
    significant_bodies.push(Body::new(
        -100.6,
        50.6,
        0.0,
        0.0,
        1000.0,
        true,
        FColor::BLUE,
    ));
    significant_bodies.push(Body::new(100.6, 150.6, 0.0, 0.0, 1000.0, true, FColor::RED));
    significant_bodies.push(Body::new(
        -300.6,
        100.6,
        0.0,
        0.0,
        1400.0,
        true,
        FColor::GREY,
    ));

    let mut pinned_bodies: Vec<Body> = vec![];

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::KeyDown {
                    timestamp: _,
                    window_id: _,
                    keycode,
                    scancode: _,
                    keymod: _,
                    repeat: _,
                    which: _,
                    raw: _,
                } => {
                    if keycode == Some(Keycode::Space) {
                        render_mode += 1;
                        if render_mode > 3 {
                            render_mode = 0;
                        }
                    }

                    if keycode == Some(Keycode::R) {
                        for body in pinned_bodies.iter_mut() {
                            body.pinned = false;
                            body.v_x = 0.0;
                            body.v_y = 0.0;
                            bodies.push(body.clone());
                        }
                    }
                }

                Event::MouseButtonDown {
                    mouse_btn,
                    timestamp: _,
                    window_id: _,
                    which: _,
                    clicks: _,
                    x,
                    y,
                } => {
                    if mouse_btn == MouseButton::Middle {
                        panning = true;
                    }

                    if mouse_btn == MouseButton::Left || mouse_btn == MouseButton::Right {
                        drawing = true;
                    }
                }

                Event::MouseMotion {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    mousestate: _,
                    x,
                    y,
                    xrel,
                    yrel,
                } => {
                    if panning {
                        pan_x += xrel;
                        pan_y += yrel;
                    }

                    if drawing {
                        for box_x in -10..10 {
                            for box_y in -10..10 {
                                bodies.push(Body::new(
                                    (((x - pan_x) / zoom) + box_x as f32 / (zoom / 9.0)).into(),
                                    (((y - pan_y) / zoom) + box_y as f32 / (zoom / 9.0)).into(),
                                    0.0,
                                    0.0,
                                    100.0,
                                    false,
                                    FColor::WHITE,
                                ));
                            }
                        }
                    }
                }

                Event::MouseButtonUp { mouse_btn, .. } => {
                    if mouse_btn == MouseButton::Middle {
                        panning = false;
                    }

                    if mouse_btn == MouseButton::Left || mouse_btn == MouseButton::Right {
                        drawing = false;
                    }
                }

                Event::MouseWheel {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    x: _,
                    y,
                    direction: _,
                    mouse_x,
                    mouse_y,
                } => {
                    // This code is not mine, I stole it
                    let zoom_factor = 1.0 + y / 20.0;
                    let new_zoom = zoom * zoom_factor;

                    pan_x = mouse_x - (mouse_x - pan_x) * (new_zoom / zoom);
                    pan_y = mouse_y - (mouse_y - pan_y) * (new_zoom / zoom);

                    zoom = new_zoom;
                }
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));

        let total_bodies = bodies.len();

        let (mut vertices, mut indices) = (vec![], vec![]);

        let mut bodies_rendered = 0;

        for body in &bodies {
            let (mut body_vertices, body_indices) = body.get_render(pan_x, pan_y, zoom);
            vertices.append(&mut body_vertices);

            indices.append(
                &mut body_indices
                    .iter()
                    .map(|i| i + 3 * bodies_rendered)
                    .collect(),
            );

            bodies_rendered += 1;
        }

        if render_mode == 0 {
            for pinned_body in pinned_bodies.iter() {
                canvas.set_draw_color(pinned_body.color);
                canvas.draw_point(Point::new(
                    ((pinned_body.x as f32 * zoom) + pan_x) as i32,
                    ((pinned_body.y as f32 * zoom) + pan_y) as i32,
                ))?;
            }
        }

        canvas.render_geometry(&vertices, None, &indices).unwrap();

        for body in &significant_bodies {
            body.render(pan_x, pan_y, zoom, &mut canvas);
        }

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        let _ = canvas.draw_debug_text(render_mode.to_string().as_str(), Point::new(100, 0));

        canvas.present();

        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));

        let mut sim_steps = 1;
        let res = 10;

        if render_mode == 2 {
            sim_steps = 20;
        } else if render_mode == 3 {
            sim_steps = 500;
        }

        sim_steps *= res;

        // sim steps per render
        for _ in 0..sim_steps {
            // Before pinning
            for body in &mut bodies.iter_mut() {
                body.x += body.v_x / res as f64;
                body.y += body.v_y / res as f64;
                body.v_x *= 0.999999;
                body.v_y *= 0.999999;
            }

            for body2 in &significant_bodies {
                bodies.par_iter_mut().for_each(|body| {
                    let delta_y = body2.y - body.y;
                    let delta_x = body2.x - body.x;
                    let dist_sq = (delta_x).powi(2) + (delta_y).powi(2);
                    // f = m1m2/r^2

                    // These are usually both sqrt so it works, collision
                    if dist_sq < body2.mass {
                        body.color = body2.color;
                        body.pinned = true;
                        body.x = body.init_x;
                        body.y = body.init_y;
                        body.v_x = 0.0;
                        body.v_y = 0.0;
                    }

                    let force = body2.mass / (dist_sq * res as f64 + 0.00000001);

                    let angle = f64::atan2(delta_y, delta_x);

                    body.v_x += force * angle.cos();
                    body.v_y += force * angle.sin();
                });
            }

            bodies = bodies
                .iter()
                .filter_map(|&body| {
                    if body.pinned {
                        pinned_bodies.push(body.clone());
                        None
                    } else {
                        Some(body)
                    }
                })
                .collect::<Vec<Body>>();
        }
    }

    Ok(())
}
