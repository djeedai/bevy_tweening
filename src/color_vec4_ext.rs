use bevy::math::Vec4;
use bevy::render::color::Color;

pub(crate) trait ColorExt {
    fn to_vec(&self) -> Vec4;
}

impl ColorExt for Color {
    fn to_vec(&self) -> Vec4 {
        Vec4::new(self.r(), self.g(), self.b(), self.a())
    }
}

pub(crate) trait Vec4Ext {
    fn to_color(&self) -> Color;
}

impl Vec4Ext for Vec4 {
    fn to_color(&self) -> Color {
        Color::rgba(self.x, self.y, self.z, self.w)
    }
}
