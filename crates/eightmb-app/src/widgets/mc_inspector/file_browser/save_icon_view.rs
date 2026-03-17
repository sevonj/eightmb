mod imp {
    use std::cell::OnceCell;
    use std::ffi::CString;
    use std::ptr;

    use adw::subclass::prelude::*;
    use eightmb::memcard::SaveIcon;
    use gtk::GLArea;
    use gtk::gdk::GLContext;
    use gtk::glib;
    use gtk::glib::Propagation;
    use gtk::prelude::GLAreaExt;
    use libloading::os::unix::Library;

    // const SHITTY_TRIANGLE: [f32; 9] = [-0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0];
    // const SHITTIER_TRIANGLE: [f32; 9] = [0.5, 0.5, 0.0, -0.5, 0.5, 0.0, 0.0, -0.5, 0.0];

    #[derive(Default)]
    pub struct SaveIconView {
        save_icon: OnceCell<SaveIcon>,
        program: OnceCell<u32>,
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
        }
    }

    impl WidgetImpl for SaveIconView {
        fn realize(&self) {
            self.parent_realize();
            let obj = self.obj();
            obj.make_current();
            if let Some(e) = obj.error() {
                println!("{e}");
                return;
            }

            let save_icon = self.save_icon.get().unwrap();

            let mut vertices: Vec<f32> = Vec::with_capacity(save_icon.vertices.len());

            for v in &save_icon.vertices {
                vertices.push(v.coords[0].x as f32 / 0x1000 as f32 * 0.3);
                vertices.push(-v.coords[0].y as f32 / 0x1000 as f32 * 0.3 - 0.5);
                vertices.push(v.coords[0].z as f32 / 0x1000 as f32 * 0.3);
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
                const VERT_SOURCE: &str = include_str!("../../../../data/shaders/basic.vs");
                const FRAG_SOURCE: &str = include_str!("../../../../data/shaders/basic.fs");
                let vert_shad = compile_shader(VERT_SOURCE, gl::VERTEX_SHADER);
                let frag_shad = compile_shader(FRAG_SOURCE, gl::FRAGMENT_SHADER);
                let program = link_program(vert_shad, frag_shad);
                self.program.set(program).expect("bind once");

                let mut vao = 0;
                let mut vbo = 0;

                gl::GenVertexArrays(1, &mut vao);
                gl::GenBuffers(1, &mut vbo);

                gl::BindVertexArray(vao);

                gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (vertices.len() * std::mem::size_of::<f32>()) as isize,
                    vertices.as_ptr() as *const _,
                    gl::STATIC_DRAW,
                );

                gl::VertexAttribPointer(
                    0,
                    3,
                    gl::FLOAT,
                    gl::FALSE,
                    3 * std::mem::size_of::<f32>() as i32,
                    ptr::null(),
                );

                gl::EnableVertexAttribArray(0);
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

            let index_count = self.save_icon.get().expect("bound").num_vertices as i32;
            unsafe {
                gl::ClearColor(0.0, 0.0, 0.0, 0.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);

                let program = *self.program.get().expect("bound");

                gl::UseProgram(program);
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
