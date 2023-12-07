

pub struct PointLight {
    pub radius: f32,
}


pub struct DirectionnalLight {
    pub direction: glam::Vec3,
    pub color: glam::Vec3,
    pub casts_shadow: bool,
}