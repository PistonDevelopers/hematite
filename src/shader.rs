use gfx::traits::{Device, DeviceExt, FactoryExt, ToSlice};
use gfx;
use vecmath::Matrix4;

static VERTEX: &'static [u8] = b"
    #version 150 core
    uniform mat4 projection, view;

    in vec2 tex_coord;
    in vec3 color, position;

    out vec2 v_tex_coord;
    out vec3 v_color;

    void main() {
        v_tex_coord = tex_coord;
        v_color = color;
        gl_Position = projection * view * vec4(position, 1.0);
    }
";

static FRAGMENT: &'static [u8] = b"
    #version 150 core
    out vec4 out_color;

    uniform sampler2D s_texture;

    in vec2 v_tex_coord;
    in vec3 v_color;

    void main() {
        vec4 tex_color = texture(s_texture, v_tex_coord);
        if(tex_color.a == 0.0) // Discard transparent pixels.
            discard;
        out_color = tex_color * vec4(v_color, 1.0);
    }
";

#[shader_param]
#[derive(Clone)]
struct ShaderParam<R: gfx::Resources> {
    pub projection: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub s_texture: gfx::shade::TextureParam<R>,
}

#[vertex_format]
#[derive(Copy)]
pub struct Vertex {
    #[name="position"]
    pub xyz: [f32; 3],
    #[name="tex_coord"]
    pub uv: [f32; 2],
    #[name="color"]
    pub rgb: [f32; 3],
}

impl Clone for Vertex {
    fn clone(&self) -> Vertex {
        *self
    }
}

pub struct Buffer<R: gfx::Resources> {
    batch: gfx::batch::RefBatch<ShaderParam<R>>,
}

pub struct Renderer<D: Device> {
    graphics: gfx::Graphics<D>,
    params: ShaderParam<D::Resources>,
    frame: gfx::Frame<D::Resources>,
    cd: gfx::ClearData,
    prog: gfx::ProgramHandle<D::Resources>,
    drawstate: gfx::DrawState
}

impl<R: gfx::device::Resources, C: gfx::device::draw::CommandBuffer<R>, D: gfx::device::Factory<R> + gfx::Device<Resources=R, CommandBuffer=C>> Renderer<D> {
    pub fn new(mut device: D, frame: gfx::Frame<D::Resources>,
               tex: gfx::TextureHandle<D::Resources>) -> Renderer<D> {
        let sampler = device.create_sampler(
                gfx::tex::SamplerInfo::new(
                    gfx::tex::FilterMethod::Scale,
                    gfx::tex::WrapMode::Tile
                )
            );

        let mut graphics = device.into_graphics();

        let params = ShaderParam {
            projection: [[0.0; 4]; 4],
            view: [[0.0; 4]; 4],
            s_texture: (tex, Some(sampler))
        };
        let prog = graphics.device.link_program(VERTEX.clone(), FRAGMENT.clone()).ok().unwrap();
        let mut drawstate = gfx::DrawState::new().depth(gfx::state::Comparison::LessEqual, true);
        drawstate.primitive.front_face = gfx::state::FrontFace::Clockwise;

        Renderer {
            graphics: graphics,
            params: params,
            frame: frame,
            cd: gfx::ClearData {
                color: [0.81, 0.8, 1.0, 1.0],
                depth: 1.0,
                stencil: 0,
            },
            prog: prog,
            drawstate: drawstate,
        }
    }

    pub fn set_projection(&mut self, proj_mat: Matrix4<f32>) {
        self.params.projection = proj_mat;
    }

    pub fn set_view(&mut self, view_mat: Matrix4<f32>) {
        self.params.view = view_mat;
    }

    pub fn clear(&mut self) {
        self.graphics.clear(self.cd, gfx::COLOR | gfx::DEPTH, &self.frame);
    }

    pub fn create_buffer(&mut self, data: &[Vertex]) -> Buffer<D::Resources> {
        let buf = self.graphics.device.create_buffer(data.len(), gfx::BufferUsage::Static);
        self.graphics.device.update_buffer(&buf, data, 0);
        let mesh = gfx::Mesh::from_format(buf, data.len() as u32);
        Buffer {
            batch: self.graphics.make_batch(
                    &self.prog,
                    self.params.clone(),
                    &mesh,
                    mesh.to_slice(gfx::PrimitiveType::TriangleList),
                    &self.drawstate
                ).unwrap()
        }
    }

    pub fn render(&mut self, buffer: &mut Buffer<D::Resources>) {
        buffer.batch.params = self.params.clone();
        self.graphics.draw(&buffer.batch, &self.frame).unwrap();
    }

    pub fn end_frame(&mut self) {
        self.graphics.end_frame();
    }
}
