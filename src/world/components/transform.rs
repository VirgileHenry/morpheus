


pub struct Transform {
    _model_mat: glam::Mat4,
}

impl Transform {
    pub fn origin() -> Transform {
        Transform { _model_mat: glam::Mat4::IDENTITY, }
    }
}