use gfx::traits::FactoryExt;
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

gfx_parameters!( ShaderParam {
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
    batch: gfx::batch::Full<ShaderParam<R>>,
}

pub struct Renderer<R: gfx::Resources, F, S> {
    factory: F,
    pub stream: S,
    params: ShaderParam<R>,
    cd: gfx::ClearData,
    prog: gfx::handle::Program<R>,
    drawstate: gfx::DrawState,
}

impl<R: gfx::Resources, F: gfx::Factory<R>, S: gfx::Stream<R>> Renderer<R, F, S> {

    pub fn new(mut factory: F, stream: S, tex: gfx::handle::Texture<R>) -> Renderer<R, F, S> {
        use std::marker::PhantomData;
        let sampler = factory.create_sampler(
                gfx::tex::SamplerInfo::new(
                    gfx::tex::FilterMethod::Scale,
                    gfx::tex::WrapMode::Tile
                )
            );

        let params = ShaderParam {
            projection: [[0.0; 4]; 4],
            view: [[0.0; 4]; 4],
            texture: (tex, Some(sampler)),
            _r: PhantomData,
        };
        let prog = factory.link_program(VERTEX.clone(), FRAGMENT.clone()).unwrap();
        let mut drawstate = gfx::DrawState::new().depth(gfx::state::Comparison::LessEqual, true);
        drawstate.primitive.front_face = gfx::state::FrontFace::Clockwise;

        Renderer {
            factory: factory,
            stream: stream,
            params: params,
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
        self.stream.clear(self.cd);
    }

    pub fn create_buffer(&mut self, data: &[Vertex]) -> Buffer<R> {
        let mesh = self.factory.create_mesh(data);
        let mut b = gfx::batch::Full::new(mesh, self.prog.clone(), self.params.clone())
                                     .unwrap();
        b.state = self.drawstate;
        Buffer { batch: b }
    }

    pub fn render(&mut self, buffer: &mut Buffer<R>) {
        buffer.batch.params = self.params.clone();
        self.stream.draw(&buffer.batch).unwrap();
    }
}
