mod photon;

use crate::pb;
use image::ImageOutputFormat;
use pb::Spec;
pub use photon::Photon;

pub trait Engine {
    // 应用一组有序的处理操作
    fn apply(&mut self, specs: &[Spec]);
    // 生成目标图片
    fn generate(self, format: ImageOutputFormat) -> Vec<u8>;
}

pub trait SpecTransform<T> {
    // 对图片使用 op 做 transform 变换
    fn transform(&mut self, op: T);
}
