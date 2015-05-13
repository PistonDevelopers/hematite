use gfx::traits::{Device, DeviceExt, FactoryExt, ToSlice};
use gfx::handle::{Texture, Program};
use gfx;
use vecmath::Matrix4;

static VERTEX: &'static [u8] = b"
    #version 150 core
    uniform mat4 u_projection, u_view;

    in vec2 at_tex_coord;
    in vec3 at_color, at_position;

    out vec2 v_tex_coord;
    out vec3 v_color;

    void main() {
        v_tex_coord = at_tex_coord;
        v_color = at_color;
        gl_Position = u_projection * u_view * vec4(at_position, 1.0);
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

gfx_parameters!( ShaderParam/Link {
    u_projection@ projection: [[f32; 4]; 4],
    u_view@ view: [[f32; 4]; 4],
    s_texture@ texture: gfx::shade::TextureParam<R>,
});

gfx_vertex!( Vertex {
    at_position@ xyz: [f32; 3],
    at_tex_coord@ uv: [f32; 2],
    at_color@ rgb: [f32; 3],
});


pub struct Buffer<R: gfx::Resources> {
    batch: gfx::batch::RefBatch<ShaderParam<R>>,
}

pub struct Renderer<D: Device, F: gfx::device::Factory<D::Resources>, O: gfx::Output<D::Resources>> {
    graphics: gfx::Graphics<D, F>,
    params: ShaderParam<D::Resources>,
    frame: O,
    cd: gfx::ClearData,
    prog: gfx::handle::Program<D::Resources>,
    drawstate: gfx::DrawState
}

impl<R: gfx::device::Resources, C: gfx::device::draw::CommandBuffer<R>,
    F: gfx::device::Factory<R>, D: gfx::Device<Resources=R, CommandBuffer=C>,
    O: gfx::Output<R>>
    Renderer<D, F, O> {

    pub fn new(device: D, mut factory: F, frame: O,
               tex: gfx::handle::Texture<D::Resources>) -> Renderer<D, F, O> {
        use std::marker::PhantomData;
        let sampler = factory.create_sampler(
                gfx::tex::SamplerInfo::new(
                    gfx::tex::FilterMethod::Scale,
                    gfx::tex::WrapMode::Tile
                )
            );

        let mut graphics = (device, factory).into_graphics();

        let params = ShaderParam {
            projection: [[0.0; 4]; 4],
            view: [[0.0; 4]; 4],
            texture: (tex, Some(sampler)),
            _r: PhantomData,
        };
        let prog = graphics.factory.link_program(VERTEX.clone(), FRAGMENT.clone()).ok().unwrap();
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
        let buf = self.graphics.factory.create_buffer(data.len(), gfx::BufferUsage::Static);
        self.graphics.factory.update_buffer(&buf, data, 0);
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
