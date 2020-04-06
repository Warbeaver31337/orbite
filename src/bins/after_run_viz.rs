use kiss3d::camera::Camera;
use kiss3d::context::Context;
use kiss3d::planar_camera::PlanarCamera;
use kiss3d::post_processing::PostProcessingEffect;
use kiss3d::renderer::Renderer;
use kiss3d::resource::{
    AllocationType, BufferType, Effect, GPUVec, ShaderAttribute, ShaderUniform,
};
use kiss3d::text::Font;
use kiss3d::window::{State, Window};
use nalgebra::{Matrix4, Point2, Point3};
use std::{thread, time};

use crate::{lib::write::Data, utils::io};

// Custom renderers are used to allow rendering objects that are not necessarily
// represented as meshes. In this example, we will render a large, growing, point cloud
// with a color associated to each point.

// Writing a custom renderer requires the main loop to be
// handled by the `State` trait instead of a `while window.render()`
// like other examples.

struct AppState {
    point_cloud_renderer: PointCloudRenderer,
    iteration: usize,
    data: Vec<Data>,
    nb_part: usize,
}

impl State for AppState {
    //! Return the custom renderer that will be called at each
    // render loop.
    fn cameras_and_effect_and_renderer(
        &mut self,
    ) -> (
        Option<&mut dyn Camera>,
        Option<&mut dyn PlanarCamera>,
        Option<&mut dyn Renderer>,
        Option<&mut dyn PostProcessingEffect>,
    ) {
        (None, None, Some(&mut self.point_cloud_renderer), None)
    }

    fn step(&mut self, window: &mut Window) {
        //! Actions called at each step of rendering

        // Erase all previous points
        self.point_cloud_renderer = PointCloudRenderer::new(2.0);

        // Get current state of simulation
        let data = self.data.get(self.iteration).unwrap();

        // Iterate over all particules
        for i in 0..self.nb_part {
            let position: Point3<f32> = Point3::new(
                *data.positions.get(i).unwrap().get(0).unwrap() as f32,
                *data.positions.get(i).unwrap().get(1).unwrap() as f32,
                *data.positions.get(i).unwrap().get(2).unwrap() as f32,
            );
            self.point_cloud_renderer
                .push(position, Point3::new(1., 1., 1.));
        }

        let text = format!("Iteration: {}", self.iteration);
        window.draw_text(
            &text,
            &Point2::new(0.0, 20.0),
            60.0,
            &Font::default(),
            &Point3::new(1.0, 1.0, 1.0),
        );

        let sleep_time = time::Duration::from_millis(60);
        thread::sleep(sleep_time);

        self.iterate()
    }
}

impl AppState {
    fn iterate(&mut self) {
        self.iteration += 1;
    }
}

/// Structure which manages the display of long-living points.
struct PointCloudRenderer {
    shader: Effect,
    pos: ShaderAttribute<Point3<f32>>,
    color: ShaderAttribute<Point3<f32>>,
    proj: ShaderUniform<Matrix4<f32>>,
    view: ShaderUniform<Matrix4<f32>>,
    colored_points: GPUVec<Point3<f32>>,
    point_size: f32,
}

impl PointCloudRenderer {
    /// Creates a new points renderer.
    fn new(point_size: f32) -> PointCloudRenderer {
        let mut shader = Effect::new_from_str(VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC);

        shader.use_program();

        PointCloudRenderer {
            colored_points: GPUVec::new(Vec::new(), BufferType::Array, AllocationType::StreamDraw),
            pos: shader.get_attrib::<Point3<f32>>("position").unwrap(),
            color: shader.get_attrib::<Point3<f32>>("color").unwrap(),
            proj: shader.get_uniform::<Matrix4<f32>>("proj").unwrap(),
            view: shader.get_uniform::<Matrix4<f32>>("view").unwrap(),
            shader,
            point_size,
        }
    }

    fn push(&mut self, point: Point3<f32>, color: Point3<f32>) {
        if let Some(colored_points) = self.colored_points.data_mut() {
            colored_points.push(point);
            colored_points.push(color);
        }
    }
}

impl Renderer for PointCloudRenderer {
    /// Actually draws the points.
    fn render(&mut self, pass: usize, camera: &mut dyn Camera) {
        if self.colored_points.len() == 0 {
            return;
        }

        self.shader.use_program();
        self.pos.enable();
        self.color.enable();

        camera.upload(pass, &mut self.proj, &mut self.view);

        self.color.bind_sub_buffer(&mut self.colored_points, 1, 1);
        self.pos.bind_sub_buffer(&mut self.colored_points, 1, 0);

        let ctxt = Context::get();
        ctxt.point_size(self.point_size);
        ctxt.draw_arrays(Context::POINTS, 0, (self.colored_points.len() / 2) as i32);

        self.pos.disable();
        self.color.disable();
    }
}

const VERTEX_SHADER_SRC: &str = "#version 100
    attribute vec3 position;
    attribute vec3 color;
    varying   vec3 Color;
    uniform   mat4 proj;
    uniform   mat4 view;
    void main() {
        gl_Position = proj * view * vec4(position, 1.0);
        Color = color;
    }";

const FRAGMENT_SHADER_SRC: &str = "#version 100
    #ifdef GL_FRAGMENT_PRECISION_HIGH
    precision highp float;
    #else
    precision mediump float;
    #endif

    varying vec3 Color;
    void main() {
        gl_FragColor = vec4(Color, 1.0);
    }";
pub fn main() {
    println!("Please enter the path of the folder of the simulation data:");
    let sim_folder_path = io::get_user_input_from_stdout();

    println!("Readindg simulation data...");
    let c: usize = io::read_nb_iter(&sim_folder_path);
    let sim_data = io::read_sim_data(c, &sim_folder_path);
    //TBC...

    let window = Window::new("orbite simulation");

    let nb_part = sim_data.get(0).unwrap().positions.len();
    let app = AppState {
        point_cloud_renderer: PointCloudRenderer::new(4.0),
        iteration: 0,
        data: sim_data,
        nb_part: nb_part,
    };

    window.render_loop(app)
}
