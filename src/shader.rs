use vecmath::Matrix4;
use gfx;
use gfx::{Device, DeviceHelper, ToSlice};
use device;
use device::draw::CommandBuffer;
use render;

static VERTEX: gfx::ShaderSource<'static> = shaders! {
GLSL_120: b"
    #version 120
    uniform mat4 projection, view;

    attribute vec2 tex_coord;
    attribute vec3 color, position;

    varying vec2 v_tex_coord;
    varying vec3 v_color;

    void main() {
        v_tex_coord = tex_coord;
        v_color = color;
        gl_Position = projection * view * vec4(position, 1.0);
    }
"
GLSL_150: b"
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
"
};

static FRAGMENT: gfx::ShaderSource<'static> = shaders!{
GLSL_120: b"
    #version 120

    uniform sampler2D s_texture;

    varying vec2 v_tex_coord;
    varying vec3 v_color;

    void main() {
        vec4 tex_color = texture2D(s_texture, v_tex_coord);
        if(tex_color.a == 0.0) // Discard transparent pixels.
            discard;
        gl_FragColor = tex_color * vec4(v_color, 1.0);
    }
"
GLSL_150: b"
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
"
};

#[shader_param(Program)]
pub struct ShaderParam {
    pub projection: [[f32, ..4], ..4],
    pub view: [[f32, ..4], ..4],
    pub s_texture: gfx::shade::TextureParam,
}

#[vertex_format]
pub struct Vertex {
    #[name="position"]
    pub xyz: [f32, ..3],
    #[name="tex_coord"]
    pub uv: [f32, ..2],
    #[name="color"]
    pub rgb: [f32, ..3],
}

impl Clone for Vertex {
    fn clone(&self) -> Vertex {
        *self
    }
}

pub struct Buffer {
    buf: gfx::BufferHandle<Vertex>,
    batch: render::batch::RefBatch<_ShaderParamLink, ShaderParam>
}

pub struct Renderer<D: Device<C>, C: CommandBuffer> {
    graphics: gfx::Graphics<D, C>,
    params: ShaderParam,
    frame: gfx::Frame,
    cd: gfx::ClearData,
    prog: device::Handle<u32, device::shade::ProgramInfo>,
    drawstate: gfx::DrawState
}

impl<D: Device<C>, C: CommandBuffer> Renderer<D, C> {
    pub fn new(mut device: D, frame: gfx::Frame, tex: gfx::TextureHandle) -> Renderer<D, C> {
        let sampler = device.create_sampler(gfx::tex::SamplerInfo::new(gfx::tex::Scale, gfx::tex::Tile));
        let mut graphics = gfx::Graphics::new(device);

        let params = ShaderParam {
            projection: [[0.0, ..4], ..4],
            view: [[0.0, ..4], ..4],
            s_texture: (tex, Some(sampler))
        };
        let prog = graphics.device.link_program(VERTEX.clone(), FRAGMENT.clone()).unwrap();
        let mut drawstate = gfx::DrawState::new().depth(gfx::state::LessEqual, true);
        drawstate.primitive.front_face = gfx::state::Clockwise;

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

    pub fn create_buffer(&mut self, data: &[Vertex]) -> Buffer {
        let buf = self.graphics.device.create_buffer(data.len(), gfx::UsageStatic);
        self.graphics.device.update_buffer(buf, data, 0);
        let mesh = gfx::Mesh::from_format(buf, data.len() as u32);
        Buffer {
            buf: buf,
            batch: self.graphics.make_batch(&self.prog, &mesh, mesh.to_slice(gfx::TriangleList),
                                            &self.drawstate).unwrap()
        }
    }

    pub fn delete_buffer(&mut self, buf: Buffer) {
        self.graphics.device.delete_buffer(buf.buf);
    }

    pub fn render(&mut self, buffer: Buffer) {
        self.graphics.draw(&buffer.batch, &self.params, &self.frame);
    }

    pub fn end_frame(&mut self) {
        self.graphics.end_frame();
    }
}
