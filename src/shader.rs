use gfx::traits::FactoryExt;
use gfx::pso::DataLink;
use gfx;
use vecmath::{self, Matrix4};

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

gfx_pipeline!( pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    transform: gfx::Global<[[f32; 4]; 4]> = "u_projection",
    view: gfx::Global<[[f32; 4]; 4]> = "u_view",
    color: gfx::TextureSampler<[f32; 4]> = "s_texture",
    out_color: gfx::RenderTarget<gfx::format::Srgba8> = "out_color",
    out_depth: gfx::DepthTarget<gfx::format::DepthStencil> = 
        gfx::preset::depth::LESS_EQUAL_WRITE,
});

gfx_vertex_struct!( Vertex {
    xyz: [f32; 3] = "at_position",
    uv: [f32; 2] = "at_tex_coord",
    rgb: [f32; 3] = "at_color",
});


pub struct Renderer<R: gfx::Resources, F: gfx::Factory<R>, C: gfx::CommandBuffer<R>> {
    factory: F,
    pub pipe: gfx::PipelineState<R, pipe::Meta>,
    data: pipe::Data<R>,
    encoder: gfx::Encoder<R, C>,
    clear_color: [f32; 4],
    clear_depth: f32,
    clear_stencil: u8,
    slice: gfx::Slice<R>,
}

impl<R: gfx::Resources, F: gfx::Factory<R>, C: gfx::CommandBuffer<R>> Renderer<R, F, C> {

    pub fn new(mut factory: F, encoder: gfx::Encoder<R, C>, target: gfx::handle::RenderTargetView<R, gfx::format::Srgba8>,
        depth: gfx::handle::DepthStencilView<R, (gfx::format::D24_S8, gfx::format::Unorm)>, 
        tex: gfx::handle::Texture<R, gfx::format::R8_G8_B8_A8>) -> Renderer<R, F, C> {

        let sampler = factory.create_sampler(
                gfx::tex::SamplerInfo::new(
                    gfx::tex::FilterMethod::Scale,
                    gfx::tex::WrapMode::Tile
                )
            );

        let texture_view = factory.view_texture_as_shader_resource::<gfx::format::Rgba8>(
            &tex, (0, 0), gfx::format::Swizzle::new()).unwrap();

        let prog = factory.link_program(VERTEX, FRAGMENT).unwrap();

        let mut rasterizer = gfx::state::Rasterizer::new_fill(gfx::state::CullFace::Back);
        rasterizer.front_face = gfx::state::FrontFace::Clockwise;
        let pipe = factory.create_pipeline_from_program(&prog, gfx::Primitive::TriangleList, 
            rasterizer, pipe::new()).unwrap();

        let (vbuf, slice) = factory.create_vertex_buffer(&[]);

        let data = pipe::Data {
            vbuf: vbuf,
            transform: vecmath::mat4_id(),
            view: vecmath::mat4_id(),
            color: (texture_view, sampler),
            out_color: target,
            out_depth: depth,
        };

        Renderer {
            factory: factory,
            pipe: pipe,
            data: data,
            encoder: encoder,
            clear_color: [0.81, 0.8, 1.0, 1.0],
            clear_depth: 1.0,
            clear_stencil: 0,
            slice: slice,
        }
    }

    pub fn set_projection(&mut self, proj_mat: Matrix4<f32>) {
        self.data.transform = proj_mat;
    }

    pub fn set_view(&mut self, view_mat: Matrix4<f32>) {
        self.data.view = view_mat;
    }

    pub fn clear(&mut self) {
        self.encoder.clear(&self.data.out_color, self.clear_color);
        self.encoder.clear_depth(&self.data.out_depth, self.clear_depth);
        self.encoder.clear_stencil(&self.data.out_depth, self.clear_stencil);
    }

    pub fn flush<D: gfx::Device<Resources=R, CommandBuffer=C> + Sized>(&mut self, device: &mut D) {
        self.encoder.flush(device);
    }

    pub fn create_buffer(&mut self, data: &[Vertex]) -> gfx::handle::Buffer<R, Vertex> {
        let (vbuf, slice) = self.factory.create_vertex_buffer(data);
        self.slice = slice;

        vbuf
    }

    pub fn render(&mut self, buffer: &mut gfx::handle::Buffer<R, Vertex>) {
        self.data.vbuf = buffer.clone();
        self.slice.end = buffer.len() as u32;
        self.encoder.draw(&self.slice, &self.pipe, &self.data);
    }
}
