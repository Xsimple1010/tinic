use super::{
    frame_buffer::FrameBuffer,
    gl::gl::{
        self, DEPTH_ATTACHMENT, DEPTH_COMPONENT24, DEPTH_STENCIL_ATTACHMENT, DEPTH24_STENCIL8,
        types::{GLint, GLuint},
    },
    gl_buffer::GlBuffer,
    render_buffer::RenderBuffer,
    shader::Shader,
    shader_program::ShaderProgram,
    texture::Texture2D,
    vertex::{GlVertex, new_vertex},
    vertex_array::VertexArray,
};
use crate::raw_texture::RawTextureData;
use glutin::prelude::GlDisplay;
use retro_core::av_info::{AvInfo, Geometry};
use std::{ffi::CString, mem::size_of, sync::atomic::Ordering};
use std::{rc::Rc, sync::Arc};
use tinic_generics::error_handle::{ErrorHandle, TinicResult};

pub struct Render {
    _program: ShaderProgram,
    _texture: Option<Texture2D>,
    _i_pos: GLint,
    _i_tex_pos: GLint,
    _u_tex: GLint,
    _vao: Option<VertexArray>,
    _vbo: Option<GlBuffer>,
    _fbo: Option<FrameBuffer>,
    _rbo: Option<RenderBuffer>,
    fbo_width: i32,
    fbo_height: i32,
    gl: Rc<gl::Gl>,
}

impl Render {
    /// Chamado ANTES de load_game.
    /// Cria apenas shaders e o loader GL.
    /// VAO, VBO, FBO e textura são criados em init_framebuffer().
    pub fn new<D: GlDisplay>(gl_display: D) -> Result<Render, ErrorHandle> {
        let vertex_shader_src = "
            #version 330 core
            in vec2 i_pos;
            in vec2 i_tex_pos;

            out vec2 f_t_pos;

            void main() {
                f_t_pos = i_tex_pos;
                gl_Position = vec4(i_pos, 0.0, 1.0);
            }
        ";

        let fragment_shader_src = "
            #version 330 core
            in vec2 f_t_pos;

            out vec4 FragColor;

            uniform sampler2D u_tex;

            void main() {
                FragColor = texture2D(u_tex, f_t_pos);
            }
        ";

        let gl = Rc::new(gl::Gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap();
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        }));

        let vertex_shader = Shader::new(gl::VERTEX_SHADER, vertex_shader_src, gl.clone())?;
        let frag_shader = Shader::new(gl::FRAGMENT_SHADER, fragment_shader_src, gl.clone())?;
        let program = ShaderProgram::new(&[vertex_shader, frag_shader], gl.clone())?;

        let i_pos = program.get_attribute("i_pos");
        let i_tex_pos = program.get_attribute("i_tex_pos");
        let u_tex = program.get_uniform("u_tex");

        Ok(Render {
            _program: program,
            _texture: None,
            _i_pos: i_pos,
            _i_tex_pos: i_tex_pos,
            _u_tex: u_tex,
            _vao: None,
            _vbo: None,
            _fbo: None,
            _rbo: None,
            fbo_width: 0,
            fbo_height: 0,
            gl,
        })
    }

    /// Chamado APÓS load_game.
    /// av_info já tem max_width, max_height, depth e stencil
    /// setados pelo core via SET_HW_RENDER.
    pub fn init_framebuffer(&mut self, av_info: &Arc<AvInfo>) -> TinicResult<()> {
        let g_api = &av_info.video.graphic_api;
        let geo = &av_info.video.geometry;

        let fbo_width = geo.max_width.load(Ordering::SeqCst) as i32;
        let fbo_height = geo.max_height.load(Ordering::SeqCst) as i32;

        let vao = VertexArray::new(self.gl.clone());
        let vbo = GlBuffer::new(gl::ARRAY_BUFFER, self.gl.clone());
        let fbo = FrameBuffer::new(self.gl.clone());
        let texture = Texture2D::new(av_info, self.gl.clone())?;

        fbo.bind();
        fbo.attach_texture(&texture);

        let mut rbo: Option<RenderBuffer> = None;

        if g_api.depth.load(Ordering::SeqCst) && g_api.stencil.load(Ordering::SeqCst) {
            let new_rbo = RenderBuffer::new(self.gl.clone());
            new_rbo.bind();
            new_rbo.storage(DEPTH24_STENCIL8, fbo_width, fbo_height);
            fbo.attach_render_buffer(DEPTH_STENCIL_ATTACHMENT, new_rbo.get_id());
            rbo = Some(new_rbo);
        } else if g_api.depth.load(Ordering::SeqCst) {
            let new_rbo = RenderBuffer::new(self.gl.clone());
            new_rbo.bind();
            new_rbo.storage(DEPTH_COMPONENT24, fbo_width, fbo_height);
            fbo.attach_render_buffer(DEPTH_ATTACHMENT, new_rbo.get_id());
            rbo = Some(new_rbo);
        }

        if let Some(r) = &rbo {
            r.un_bind();
        }

        let status = unsafe { self.gl.CheckFramebufferStatus(gl::FRAMEBUFFER) };

        if status != gl::FRAMEBUFFER_COMPLETE {
            fbo.un_bind();
            return Err(ErrorHandle::new(&format!("FBO incompleto: {:#X}", status)));
        }

        fbo.un_bind();

        // Salva o ID do FBO para o callback get_current_framebuffer
        av_info
            .video
            .graphic_api
            .fbo
            .write()?
            .replace(fbo.get_id() as usize);

        self._vao = Some(vao);
        self._vbo = Some(vbo);
        self._fbo = Some(fbo);
        self._rbo = rbo;
        self._texture = Some(texture);
        self.fbo_width = fbo_width;
        self.fbo_height = fbo_height;

        Ok(())
    }

    /// Deve ser chamado ANTES de retro_run() no modo HW.
    pub fn prepare_for_core(&self) {
        let fbo = match &self._fbo {
            Some(fbo) => fbo,
            None => return,
        };

        if self.fbo_width == 0 || self.fbo_height == 0 {
            return;
        }

        unsafe {
            self.gl.BindFramebuffer(gl::FRAMEBUFFER, fbo.get_id());
            self.gl.Viewport(0, 0, self.fbo_width, self.fbo_height);
        }
    }

    fn blit_fbo_to_screen(&self, win_width: i32, win_height: i32) {
        let fbo = match &self._fbo {
            Some(fbo) => fbo,
            None => return,
        };

        unsafe {
            self.gl.BindFramebuffer(gl::READ_FRAMEBUFFER, fbo.get_id());
            self.gl.BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);

            self.gl.BlitFramebuffer(
                0,
                0,
                self.fbo_width,
                self.fbo_height,
                0,
                0,
                win_width,
                win_height,
                gl::COLOR_BUFFER_BIT,
                gl::NEAREST,
            );

            self.gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    fn refresh_vertex(
        &self,
        geo: &Geometry,
        origin_w: f32,
        origin_h: f32,
        window_w: i32,
        window_h: i32,
    ) {
        let vao = match &self._vao {
            Some(vao) => vao,
            None => return,
        };
        let vbo = match &self._vbo {
            Some(vbo) => vbo,
            None => return,
        };

        let vertex = new_vertex(geo, window_w as f32, window_h as f32, origin_w, origin_h);

        vao.bind();
        vbo.bind();

        vbo.set_data(vertex);
        vao.set_attribute::<GlVertex>(self._i_pos as GLuint, 2, 0);
        vao.set_attribute::<GlVertex>(self._i_tex_pos as GLuint, 2, (size_of::<f32>() * 2) as i32);

        vao.un_bind();
        vbo.un_bind();
    }

    pub fn draw_new_frame(
        &self,
        texture: &RawTextureData,
        geo: &Geometry,
        win_width: i32,
        win_height: i32,
    ) {
        let vao = match &self._vao {
            Some(vao) => vao,
            None => return,
        };

        unsafe {
            self.gl.BindFramebuffer(gl::FRAMEBUFFER, 0);
            self.gl.Viewport(0, 0, win_width, win_height);
            self.gl.ClearColor(0.0, 0.0, 0.0, 1.0);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);

            if texture.is_hw {
                self.blit_fbo_to_screen(win_width, win_height);
                return;
            }

            let texture2d = match &self._texture {
                Some(tex) => tex,
                None => return,
            };

            self.refresh_vertex(
                geo,
                texture.width as f32,
                texture.height as f32,
                win_width,
                win_height,
            );

            texture2d.push(texture);
            self._program.use_program();
            texture2d.active();

            vao.bind();
            self.gl.DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
            vao.un_bind();
            self._program.un_use_program();
        }
    }
}
