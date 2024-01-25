use glow::HasContext;

pub fn setup() -> (
    sdl2::Sdl,
    sdl2::VideoSubsystem,
    sdl2::video::Window,
    glow::Context,
    sdl2::video::GLContext,
) {
    let sdl = sdl2::init().unwrap();

    let video = sdl.video().unwrap();

    let window = video
        .window("emulator", 400, 400)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .unwrap();

    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_flags().forward_compatible().set();

    let gl_context = window.gl_create_context().unwrap();
    unsafe {
        let gl = glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _);
        (sdl, video, window, gl, gl_context)
    }
}

pub unsafe fn create_program(
    gl: &glow::Context,
    vert_shader: &str,
    frag_shader: &str,
) -> glow::Program {
    let program = gl.create_program().unwrap();
    let shaders_src = [
        (glow::VERTEX_SHADER, vert_shader),
        (glow::FRAGMENT_SHADER, frag_shader),
    ];

    let mut shaders = vec![];

    for (shader_type, src) in shaders_src.iter() {
        let shader = gl.create_shader(*shader_type).unwrap();
        gl.shader_source(shader, src);
        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            let shader_type = if *shader_type == glow::VERTEX_SHADER {
                "vertex shader"
            } else {
                "fragment shader"
            };
            panic!(
                "error on {}:{}",
                shader_type,
                gl.get_shader_info_log(shader)
            );
        }
        gl.attach_shader(program, shader);
        shaders.push(shader);
    }

    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        panic!("{}", gl.get_program_info_log(program));
    }

    for shader in shaders {
        gl.detach_shader(program, shader);
        gl.delete_shader(shader);
    }

    return program;
}
