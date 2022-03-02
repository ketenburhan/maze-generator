use std::collections::HashSet;

use anyhow::Result;
use pixels::{Pixels, SurfaceTexture};
use rand::Rng;
use winit::{
    dpi::PhysicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

const CELL_SIZE: u32 = 20;

const COLS: u32 = 30;
const ROWS: u32 = 30;

const CELL_COLOR: [u8; 4] = [0x99, 0x99, 0xff, 0xff];
const VISITED_COLOR: [u8; 4] = [0xff, 0x99, 0x99, 0xff];
const WALL_COLOR: [u8; 4] = [0xff, 0xff, 0xff, 0xff];

const WIN_WIDTH: u32 = COLS * CELL_SIZE;
const WIN_HEIGHT: u32 = ROWS * CELL_SIZE;

fn main() -> Result<()> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = PhysicalSize::new(WIN_WIDTH as f64, WIN_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("KTN_FLOATING maze generator")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)?
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIN_WIDTH, WIN_HEIGHT, surface_texture)?
    };

    let mut world = World::new();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| eprintln!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw
            let request_redraw = world.update();
            if request_redraw {
                window.request_redraw();
            }
        }
    });
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum WallOrientation {
    Vertical,
    Horizontal,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Wall {
    orientation: WallOrientation,
    x: usize,
    y: usize,
}

#[derive(Clone, Default, Debug)]
struct World {
    visited: HashSet<(usize, usize)>,
    stack: Vec<(usize, usize)>,
    removed_walls: HashSet<Wall>,
}

impl World {
    fn new() -> Self {
        let mut visited = HashSet::new();
        visited.insert((0, 0));
        Self {
            visited,
            stack: vec![(0, 0)],
            ..Default::default()
        }
    }
    fn update(&mut self) -> bool {
        let mut rng = rand::thread_rng();

        let last = self.stack.last();
        if last.is_none() {
            return false;
        }
        let &(current_x, current_y) = last.unwrap();
        let mut neighbours = vec![];

        if current_x > 0 {
            let neighbour = (current_x - 1, current_y);
            if !self.visited.contains(&neighbour) {
                neighbours.push((neighbour, WallOrientation::Vertical));
            }
        }
        if current_x + 1 < COLS as usize {
            let neighbour = (current_x + 1, current_y);
            if !self.visited.contains(&neighbour) {
                neighbours.push((neighbour, WallOrientation::Vertical));
            }
        }
        if current_y > 0 {
            let neighbour = (current_x, current_y - 1);
            if !self.visited.contains(&neighbour) {
                neighbours.push((neighbour, WallOrientation::Horizontal));
            }
        }
        if current_y + 1 < ROWS as usize {
            let neighbour = (current_x, current_y + 1);
            if !self.visited.contains(&neighbour) {
                neighbours.push((neighbour, WallOrientation::Horizontal));
            }
        }

        // println!("{neighbours:#?}\n{current_x} {current_y}");

        if neighbours.len() == 0 {
            self.stack.pop();
            return false;
        }

        let next_index = rng.gen_range(0..neighbours.len());
        let ((next_x, next_y), orientation) = &neighbours[next_index];
        let next = (*next_x, *next_y);

        self.visited.insert(next);
        self.stack.push(next);
        self.removed_walls.insert(Wall {
            orientation: orientation.clone(),
            x: *next_x.min(&current_x),
            y: *next_y.min(&current_y),
        });
        true
    }
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = i as u32 % WIN_WIDTH;
            let y = i as u32 / WIN_WIDTH;

            let rgba = if x > 0 && x % CELL_SIZE == 0 {
                if !self.removed_walls.contains(&Wall {
                    orientation: WallOrientation::Vertical,
                    x: (x / CELL_SIZE) as usize - 1,
                    y: (y / CELL_SIZE) as usize,
                }) {
                    WALL_COLOR
                } else {
                    VISITED_COLOR
                }
            } else if y > 0 && y % CELL_SIZE == 0 {
                if !self.removed_walls.contains(&Wall {
                    orientation: WallOrientation::Horizontal,
                    x: (x / CELL_SIZE) as usize,
                    y: (y / CELL_SIZE) as usize - 1,
                }) {
                    WALL_COLOR
                } else {
                    VISITED_COLOR
                }
            } else {
                let (col, row) = (x / CELL_SIZE, y / CELL_SIZE);
                if self.visited.contains(&(col as usize, row as usize)) {
                    VISITED_COLOR
                } else {
                    CELL_COLOR
                }
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}
