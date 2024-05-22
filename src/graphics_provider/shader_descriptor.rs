#[derive(Debug, Clone)]
pub struct ShaderDescriptor {
    pub file: &'static str,
    pub vertex_shader: &'static str,
    pub fragment_shader: &'static str,
}

