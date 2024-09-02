#[macro_use]
extern crate glium;
mod support;
mod sim;
use rayon::prelude::*;
use glium::index::PrimitiveType;
use glium::{Display, DrawParameters, PolygonMode, Surface};
use glutin::surface::WindowSurface;
use std::fs;
use support::{ApplicationContext, State};

const WIDTH: usize = 100;
const HEIGHT: usize = 50;
const CELLS: usize = WIDTH * HEIGHT;

fn main() {
    State::<Application>::run_loop();
}

#[derive(Copy, Clone)]
struct Vertex {
    pub in_color: [f32; 3],
}
implement_vertex!(Vertex, in_color);

fn prepare_vertex_data(density: &sim::Density, dim_sim: (usize, usize)) -> Vec<Vertex> {
    (0..dim_sim.0*dim_sim.1).into_par_iter().map(|i| Vertex {
        in_color: [density.r[i], density.g[i], density.b[i]],
    }).collect::<Vec<Vertex>>()
}
fn create_index_buffer(dim_sim: (usize, usize)) -> Vec<u16> {
    let mut indices = Vec::new();
    for y in 0..dim_sim.1 - 1 {
        for x in 0..dim_sim.0 - 1 {
            let i = (y * dim_sim.0 + x) as u16;
            indices.push(i);
            indices.push(i + 1);
            indices.push(i + dim_sim.0 as u16);

            indices.push(i + 1);
            indices.push(i + 1 + dim_sim.0 as u16);
            indices.push(i + dim_sim.0 as u16);
        }
    }
    indices
}
struct Application {
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub index_buffer: glium::IndexBuffer<u16>,
    pub program: glium::Program,
    pub flow_box: sim::FlowBox<CELLS>,
    pub iter: u128,
}

impl ApplicationContext for Application {
    const WINDOW_TITLE: &'static str = "Fluid Sim";

    fn new(display: &Display<WindowSurface>) -> Self {
        let flow_box = sim::FlowBox::<CELLS>::init(WIDTH, HEIGHT);

        let vertex_buffer = glium::VertexBuffer::new(
            display,
            &prepare_vertex_data(&flow_box.density, (WIDTH, HEIGHT)),
        )
        .unwrap();

        // building the index buffer
        let index_buffer = glium::IndexBuffer::new(
            display,
            PrimitiveType::TrianglesList,
            &create_index_buffer((WIDTH, HEIGHT)),
        )
        .unwrap();

        let vertex_shader_src =
            fs::read_to_string("shaders/vert_shader.glsl").expect("unable to load vertex shader.");
        // let _tessellation_control_shader_src = fs::read_to_string("shaders/tcs_shader.glsl")
        //     .expect("unable to load tessellation control shader.");
        // let _tessellation_evaluation_shader_src = fs::read_to_string("shaders/tes_shader.glsl")
        //     .expect("unable to load tessellation evaluation shader.");
        let fragment_shader_src = fs::read_to_string("shaders/frag_shader.glsl")
            .expect("unable to load fragment shader.");
        // compiling shaders and linking them together
        let program = program!(display,
            450 => {
                vertex: vertex_shader_src.as_str(),
                fragment: fragment_shader_src.as_str(),
            },
        )
        .unwrap();
        Self {
            vertex_buffer,
            index_buffer,
            program,
            flow_box,
            iter: 0,
        }
    }

    fn draw_frame(&mut self, display: &Display<WindowSurface>) {
        let pos = (WIDTH / 2, HEIGHT / 2);
        let angle = self.iter as f32 / 60.;
        
        self.flow_box.add_fluid_velocity_angle_mag(pos.0, pos.1, angle, 90000.0);
        self.flow_box.add_fluid_density(
            pos.0,
            pos.1,
            ((angle* 3.0) % 1.0, angle % 1.0,  (angle* 2.0) % 1.0),
        );

        self.flow_box.step(1.0/30.0);
        self.vertex_buffer.write(&prepare_vertex_data(&self.flow_box.density, (WIDTH, HEIGHT)));
        let mut frame = display.draw();
                
        // Now we can draw the triangle
        frame.clear_color(0.0, 0.0, 0.0, 0.0);
        frame
            .draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &self.program,
                &uniform! {sim_dim: [WIDTH as i32, HEIGHT as i32]},
                &DrawParameters {
                    polygon_mode: PolygonMode::Fill,
                    ..Default::default()
                },
            )
            .unwrap();
        frame.finish().unwrap();
            
        self.iter = self.iter.wrapping_add(1);
    }
}
