mod imp {
    use std::cell::OnceCell;
    use std::ffi::CString;
    use std::mem::size_of;
    use std::ptr;
    use std::time::Instant;

    use adw::subclass::prelude::*;
    use eightmb::memcard::SaveIcon;
    use glam::Mat4;
    use glam::vec3;
    use gtk::GLArea;
    use gtk::gdk::GLContext;
    use gtk::glib;
    use gtk::glib::Propagation;
    use gtk::prelude::{GLAreaExt, WidgetExt, WidgetExtManual};
    use libloading::os::unix::Library;

    #[derive(Default)]
    pub struct SaveIconView {
        save_icon: OnceCell<SaveIcon>,
        program: OnceCell<u32>,
        texture: OnceCell<u32>,
        start_time: OnceCell<Instant>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SaveIconView {
        const NAME: &'static str = "SaveIconView";
        type Type = super::SaveIconView;
        type ParentType = GLArea;
    }

    impl ObjectImpl for SaveIconView {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            obj.set_has_depth_buffer(true);

            obj.add_tick_callback(move |obj, _| {
                obj.queue_render();
                glib::ControlFlow::Continue
            });

            obj.connect_resize(|area, width, height| {
                area.make_current();
                unsafe {
                    gl::Viewport(0, 0, width, height);
                }
            });
        }
    }

    impl WidgetImpl for SaveIconView {
        fn realize(&self) {
            self.parent_realize();
            let obj = self.obj();
            obj.make_current();
            if let Some(e) = obj.error() {
                println!("'{e}'");
                return;
            }

            let save_icon = self.save_icon.get().unwrap();
            self.start_time.set(Instant::now()).expect("bind once");

            // XYZ_UV_RGBA
            let mut vbuf: Vec<f32> = Vec::with_capacity(save_icon.vertices.len());
            for v in &save_icon.vertices {
                vbuf.push(v.coords[0].x as f32 / 0x1000 as f32 * 0.3);
                vbuf.push(-v.coords[0].y as f32 / 0x1000 as f32 * 0.3);
                vbuf.push(v.coords[0].z as f32 / 0x1000 as f32 * 0.3);
                vbuf.push(v.u as f32 / 0x1000 as f32);
                vbuf.push(v.v as f32 / 0x1000 as f32);
                vbuf.push(v.rgba[0] as f32 / 0xff as f32);
                vbuf.push(v.rgba[1] as f32 / 0xff as f32);
                vbuf.push(v.rgba[2] as f32 / 0xff as f32);
                vbuf.push(v.rgba[3] as f32 / 0xff as f32);
            }

            let libepoxy =
                unsafe { Library::new("libepoxy.so.0") }.expect("Couldn't to get 'libepoxy.so.0'");
            epoxy::load_with(
                |symbol| match unsafe { libepoxy.get::<_>(symbol.as_bytes()) } {
                    Ok(v) => *v,
                    Err(_) => ptr::null(),
                },
            );
            gl::load_with(epoxy::get_proc_addr);

            unsafe {
                // Shaders
                const VERT_SOURCE: &str = include_str!("../../../../data/shaders/save_icon.vs");
                const FRAG_SOURCE: &str = include_str!("../../../../data/shaders/save_icon.fs");
                let vert_shad = compile_shader(VERT_SOURCE, gl::VERTEX_SHADER);
                let frag_shad = compile_shader(FRAG_SOURCE, gl::FRAGMENT_SHADER);
                let program = link_program(vert_shad, frag_shad);
                self.program.set(program).expect("bind once");

                // Vertex
                let mut vao = 0;
                let mut vbo = 0;

                gl::GenVertexArrays(1, &mut vao);
                gl::GenBuffers(1, &mut vbo);

                gl::BindVertexArray(vao);

                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (vbuf.len() * size_of::<f32>()) as isize,
                    vbuf.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );

                let stride = (3 + 2 + 4) * size_of::<f32>() as i32;
                let uv_off = 3 * size_of::<f32>() as i32;
                let rgba_off = (uv_off + 2) * size_of::<f32>() as i32;
                gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
                gl::EnableVertexAttribArray(0);
                gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, uv_off as *const _);
                gl::EnableVertexAttribArray(1);
                gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, stride, rgba_off as *const _);
                gl::EnableVertexAttribArray(2);

                // Texture
                let mut texture = 0;
                gl::GenTextures(1, &mut texture);
                gl::BindTexture(gl::TEXTURE_2D, texture);
                self.texture.set(texture).expect("bind once");

                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as i32,
                    128,
                    128,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    save_icon.texture.as_ref().as_ptr() as *const _,
                );

                // Misc
                gl::Enable(gl::DEPTH_TEST);
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            }
        }

        fn unrealize(&self) {
            self.parent_unrealize();
        }
    }

    impl GLAreaImpl for SaveIconView {
        fn render(&self, _context: &GLContext) -> Propagation {
            let obj = self.obj();

            if let Some(e) = obj.error() {
                println!("{e}");
                return Propagation::Stop;
            }

            obj.make_current();

            let t = self
                .start_time
                .get()
                .expect("bound")
                .elapsed()
                .as_secs_f32();

            let aspect = obj.width() as f32 / obj.height() as f32;
            let projection = Mat4::perspective_rh_gl(60.0_f32.to_radians(), aspect, 0.01, 10.0);

            let rotation = Mat4::from_rotation_y(-t * 0.4);
            let translation = Mat4::from_translation(vec3(0., -0.1, -0.5));
            let scale = Mat4::from_scale(vec3(0.2, 0.2, -0.2));
            let transform = projection * translation * rotation * scale;

            let index_count = self.save_icon.get().expect("bound").num_vertices as i32;
            unsafe {
                let program = *self.program.get().expect("bound");
                let location = gl::GetUniformLocation(program, c"transform".as_ptr() as _);

                gl::ClearColor(0.0, 0.0, 0.0, 0.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

                gl::UseProgram(program);

                gl::UniformMatrix4fv(location, 1, gl::FALSE, transform.as_ref() as *const f32);
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, *self.texture.get().expect("bound"));

                gl::DrawArrays(gl::TRIANGLES, 0, index_count);
            }

            Propagation::Stop
        }
    }

    impl SaveIconView {
        pub(super) fn bind(&self, save_icon: SaveIcon) {
            self.save_icon.set(save_icon).expect("bind once");
        }
    }

    unsafe fn compile_shader(source: &str, shader_type: u32) -> u32 {
        unsafe {
            let shader = gl::CreateShader(shader_type);
            gl::ShaderSource(
                shader,
                1,
                &CString::new(source).unwrap().as_ptr(),
                ptr::null(),
            );
            gl::CompileShader(shader);

            let mut result = 0;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut result);

            if result == 0 {
                let mut len = 0;
                let log = CString::from_vec_unchecked(vec![b' '; len as usize]);
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                gl::GetShaderInfoLog(shader, len, ptr::null_mut(), log.as_ptr() as *mut _);
                panic!("Couldn't compile shader: {log:?}");
            }

            shader
        }
    }

    unsafe fn link_program(vertex: u32, fragment: u32) -> u32 {
        unsafe {
            let program = gl::CreateProgram();
            gl::AttachShader(program, vertex);
            gl::AttachShader(program, fragment);
            gl::LinkProgram(program);

            let mut result = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut result);

            if result == 0 {
                panic!("Somehow program link failed");
            }

            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);

            program
        }
    }
}

use adw::subclass::prelude::ObjectSubclassIsExt;
use eightmb::memcard::SaveIcon;
use gtk::glib;
use gtk::glib::Object;

glib::wrapper! {
    pub struct SaveIconView(ObjectSubclass<imp::SaveIconView>)
        @extends gtk::GLArea, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl SaveIconView {
    pub fn new(save_icon: SaveIcon) -> Self {
        let obj: Self = Object::builder().build();
        obj.imp().bind(save_icon);
        obj
    }
}
