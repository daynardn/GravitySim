use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash};
use std::time::SystemTime;

use rayon::iter::Either;
use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseButton;
use sdl3::pixels::{Color, FColor};
use sdl3::rect::Point;
use sdl3::render::{Canvas, FPoint, Vertex};

use std::hash::Hasher;

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
    x: f32,
    y: f32,
    init_x: f32,
    init_y: f32,
    v_x: f32,
    v_y: f32,
    mass: f32,
    pinned: bool,
    color_index: u8,
}

impl Body {
    pub fn new(
        x: f32,
        y: f32,
        v_x: f32,
        v_y: f32,
        mass: f32,
        pinned: bool,
        color_index: u8,
    ) -> Self {
        Body {
            x,
            y,
            init_x: x,
            init_y: y,
            v_x,
            v_y,
            mass,
            pinned,
            color_index,
        }
    }

    pub fn get_render(
        &self,
        pan_x: f32,
        pan_y: f32,
        zoom: f32,
        color: FColor,
    ) -> (Vec<Vertex>, Vec<i32>) {
        let (vertices, indices) = generate_circle_fan(
            FPoint::new(
                (self.x as f32 * zoom) + pan_x,
                (self.y as f32 * zoom) + pan_y,
            ), // center
            self.mass.abs().sqrt() * zoom.max(0.1), // radius
            3,                                      // segments
            color,                                  // color
        );

        return (vertices, indices);
    }

    fn render(
        &self,
        pan_x: f32,
        pan_y: f32,
        zoom: f32,
        canvas: &mut Canvas<Window>,
        color: FColor,
    ) {
        let (vertices, indices) = generate_circle_fan_color_edge(
            FPoint::new(
                (self.x as f32 * zoom) + pan_x,
                (self.y as f32 * zoom) + pan_y,
            ), // center
            self.mass.abs().sqrt() as f32 * zoom.max(0.1), // radius
            30,                                            // segments
            color,                                         // color
        );

        canvas.render_geometry(&vertices, None, &indices).unwrap();
    }
}

fn apply_force(body: &mut Body, body2: &Body, res: i32) {
    let delta_y = body2.y - body.y;
    let delta_x = body2.x - body.x;
    let dist_sq = (delta_x).powi(2) + (delta_y).powi(2);
    // f = m1m2/r^2

    // These are usually both sqrt so it works, collision
    if dist_sq < body2.mass {
        body.color_index = body2.color_index;
        body.pinned = true;
        body.x = body.init_x;
        body.y = body.init_y;
        body.v_x = 0.0;
        body.v_y = 0.0;
    }

    let force = body2.mass / (dist_sq * res as f32 + 0.00000001);

    let angle = f32::atan2(delta_y, delta_x);

    body.v_x += force * angle.cos();
    body.v_y += force * angle.sin();
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl3 demo: Video", 800, 600)
        .position_centered()
        .resizable()
        .fullscreen()
        .vulkan()
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
    let mut paused = false;
    let mut pan_x = 0.0;
    let mut pan_y = 0.0;

    let mut compute_time = 0;
    let mut compute_time_total = 0;
    let mut render_time;

    let mut zoom = 1.0;

    let mut render_mode = 0;

    let res = 1.0;
    let size = (600 as f32 * res) as i32;
    for x in -size..size {
        for y in -size / 2..size / 2 {
            bodies.push(Body::new(
                (x as f32 / res).into(),
                (y as f32 / res).into(),
                0.0,
                0.0,
                100.0,
                false,
                1,
            ));
        }
    }

    let mut color_vec = [
        FColor::YELLOW,
        FColor::WHITE,
        FColor::GRAY,
        FColor::BLUE,
        FColor::RED,
        FColor::GREEN,
        FColor::MAGENTA,
    ];

    significant_bodies.push(Body::new(0.0, -500.0, 0.0, 0.0, 1000.0, true, 0));
    significant_bodies.push(Body::new(-1000.6, 50.6, 0.0, 0.0, 1000.0, true, 3));
    significant_bodies.push(Body::new(1090.6, 150.6, 0.0, 0.0, 1000.0, true, 4));
    significant_bodies.push(Body::new(-300.6, 1000.6, 0.0, 0.0, 1400.0, true, 2));

    let mut pinned_bodies: Vec<Body> = vec![];

    let mut sim_steps = 1;
    let mut sim_steps_taken = 0;
    let res = 1;

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
                    let max_keycode = 3;
                    if keycode == Some(Keycode::Left) {
                        render_mode -= 1;
                        if render_mode < 0 {
                            render_mode = max_keycode;
                        }
                    }

                    if keycode == Some(Keycode::Right) {
                        render_mode += 1;
                        if render_mode > max_keycode {
                            render_mode = 0;
                        }
                    }

                    if render_mode == 0 || render_mode == 1 {
                        sim_steps = 1;
                    } else if render_mode == 2 {
                        sim_steps = 20;
                    } else if render_mode == 3 {
                        sim_steps = 500;
                    }

                    if keycode == Some(Keycode::Space) {
                        paused = !paused;
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
                    x: _,
                    y: _,
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
                                    1,
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

        let render_start = SystemTime::now();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));

        let (mut vertices, mut indices) = (vec![], vec![]);

        for (index, body) in bodies.iter().enumerate() {
            let (mut body_vertices, body_indices) =
                body.get_render(pan_x, pan_y, zoom, color_vec[body.color_index as usize]);
            vertices.append(&mut body_vertices);

            indices.append(&mut body_indices.iter().map(|i| i + 3 * index as i32).collect());
        }

        if render_mode == 0 {
            let mut point_map: HashMap<usize, Vec<FPoint>> = HashMap::new();

            for body in significant_bodies.iter() {
                if !point_map.contains_key(&(body.color_index as usize)) {
                    point_map.insert(body.color_index as usize, vec![]);
                }
            }

            for pinned_body in pinned_bodies.iter() {
                // canvas.set_draw_color(pinned_body.color);
                point_map
                    .get_mut(&(pinned_body.color_index as usize))
                    .unwrap_or(&mut vec![])
                    .push(FPoint::new(
                        ((pinned_body.x as f32 * zoom) + pan_x) as f32,
                        ((pinned_body.y as f32 * zoom) + pan_y) as f32,
                    ));
            }

            // Iterate on bodies not map since we need the order constant to avoid z clipping
            for body in significant_bodies.iter() {
                canvas.set_draw_color(color_vec[body.color_index as usize]);

                let points = &point_map[&(body.color_index as usize)];

                let _ = canvas.draw_points(&points[..]);
            }
        }

        canvas.render_geometry(&vertices, None, &indices).unwrap();

        for body in &significant_bodies {
            body.render(
                pan_x,
                pan_y,
                zoom,
                &mut canvas,
                color_vec[body.color_index as usize],
            );
        }

        render_time = render_start.elapsed()?.as_nanos();

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        let _ = canvas.draw_debug_text(render_mode.to_string().as_str(), Point::new(100, 0));
        let _ = canvas.draw_debug_text(sim_steps.to_string().as_str(), Point::new(100, 10));

        if paused {
            let _ = canvas.draw_debug_text("||", Point::new(80, 0));
        } else {
            let _ = canvas.draw_debug_text(">", Point::new(80, 0));
        }
        let _ = canvas.draw_debug_text(
            ("Compute time: ".to_string() + &compute_time.to_string()).as_str(),
            Point::new(180, 0),
        );

        let _ = canvas.draw_debug_text(
            ("Render time: ".to_string() + &render_time.to_string()).as_str(),
            Point::new(180, 10),
        );
        let _ = canvas.draw_debug_text(
            ("Body num: ".to_string() + &bodies.len().to_string()).as_str(),
            Point::new(180, 20),
        );

        canvas.present();

        // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));

        let compute_start = SystemTime::now();
        // sim steps per render
        for _ in 0..sim_steps * res * (!paused as i32) {
            let (bodies_vec, mut pinned_vec) = bodies.par_iter_mut().partition_map(|body| {
                // Before pinning
                body.x += body.v_x / res as f32;
                body.y += body.v_y / res as f32;
                body.v_x *= 0.999999;
                body.v_y *= 0.999999;

                for body2 in &significant_bodies {
                    apply_force(body, body2, res);
                }

                if !body.pinned {
                    Either::Left(*body)
                } else {
                    Either::Right(*body)
                }
            });

            bodies = bodies_vec;
            pinned_bodies.append(&mut pinned_vec);
        }

        sim_steps_taken += 1;
        compute_time_total += compute_start.elapsed()?.as_nanos();
        if sim_steps_taken == 10 {
            println!("Compute time: {:?}", compute_time_total / sim_steps_taken);
        }

        compute_time = (compute_start.elapsed()?.as_nanos() * 10000)
            / (bodies.len().max(1) * sim_steps as usize) as u128;
    }

    Ok(())
}
