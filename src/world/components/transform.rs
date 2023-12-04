


pub struct Transform {
    model_mat: glam::Mat4,
}

impl Transform {
    pub fn origin() -> Transform {
        Transform { model_mat: glam::Mat4::IDENTITY, }
    }
}