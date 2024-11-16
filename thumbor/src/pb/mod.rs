use base64::{engine::general_purpose, Engine as _};
use photon_rs::transform::SamplingFilter;
use prost::Message;
use tracing::info;

mod abi;
pub use abi::*;

impl ImageSpec {
    /// 给 ImageSpec 添加构造方法
    pub fn new(specs: Vec<Spec>) -> Self {
        Self { specs }
    }
}

// 从 ImageSpec 生成 Base64 字符串
impl From<&ImageSpec> for String {
    fn from(image_spec: &ImageSpec) -> Self {
        let data = image_spec.encode_to_vec();
        general_purpose::URL_SAFE_NO_PAD.encode(&data)
    }
}

// 从字符串尝试生成 ImageSpec。比如 s.parse().unwrap()
impl TryFrom<&str> for ImageSpec {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let data = general_purpose::URL_SAFE_NO_PAD.decode(&value)?;
        Ok(ImageSpec::decode(&data[..])?)
    }
}

// 辅助函数, photon_rs 相应的方法里需要字符串
impl filter::Filter {
    pub fn to_str(&self) -> Option<&str> {
        match self {
            filter::Filter::Unspecified => None,
            filter::Filter::Oceanic => Some("oceanic"),
            filter::Filter::Islands => Some("islands"),
            filter::Filter::Marine => Some("marine"),
            filter::Filter::Seagreen => Some("seagreen"),
            filter::Filter::Flagblue => Some("flagblue"),
            filter::Filter::Liquid => Some("liquid"),
            filter::Filter::Diamante => Some("diamante"),
            filter::Filter::Radio => Some("radio"),
            filter::Filter::Twenties => Some("twenties"),
            filter::Filter::Rosetint => Some("rosetint"),
            filter::Filter::Mauve => Some("mauve"),
            filter::Filter::Bluechrome => Some("bluechrome"),
            filter::Filter::Vintage => Some("vintage"),
            filter::Filter::Perfume => Some("perfume"),
            filter::Filter::Serenity => Some("serenity"),
        }
    }
}

impl blend::Blend {
    pub fn to_str(&self) -> Option<&str> {
        match self {
            blend::Blend::Overlay => Some("overlay"),
            blend::Blend::Over => Some("over"),
            blend::Blend::Atop => Some("atop"),
            blend::Blend::Xor => Some("xor"),
            blend::Blend::Multiply => Some("multiply"),
            blend::Blend::Burn => Some("burn"),
            blend::Blend::SoftLight => Some("softLight"),
            blend::Blend::HardLight => Some("hardLight"),
            blend::Blend::Difference => Some("difference"),
            blend::Blend::Lighten => Some("lighten"),
            blend::Blend::Darken => Some("darken"),
            blend::Blend::Dodge => Some("dodge"),
            blend::Blend::Plus => Some("plus"),
            blend::Blend::Exclusion => Some("exclusion")
        }
    }
}

// 在我们定义的 SampleFilter 和 photon_rs 的 SampleFilter 间转换
impl From<resize::SampleFilter> for SamplingFilter {
    fn from(v: resize::SampleFilter) -> Self {
        match v {
            resize::SampleFilter::Undefined => SamplingFilter::Nearest,
            resize::SampleFilter::Nearest => SamplingFilter::Nearest,
            resize::SampleFilter::Triangle => SamplingFilter::Triangle,
            resize::SampleFilter::CatmullRom => SamplingFilter::CatmullRom,
            resize::SampleFilter::Gaussian => SamplingFilter::Gaussian,
            resize::SampleFilter::Lanczos3 => SamplingFilter::Lanczos3,
        }
    }
}

// 提供一些辅助函数, 让创建一个 spec 的过程简单一些
impl Spec {
    pub fn new_resize_seam_carve(width: u32, height: u32) -> Self {
        Self {
            data: Some(spec::Data::Resize(Resize {
                width,
                height,
                rtype: resize::ResizeType::SeamCarve as i32,
                filter: resize::SampleFilter::Undefined as i32,
            })),
        }
    }

    pub fn new_resize(width: u32, height: u32, filter: resize::SampleFilter) -> Self {
        Self {
            data: Some(spec::Data::Resize(Resize {
                width,
                height,
                rtype: resize::ResizeType::Normal as i32,
                filter: filter as i32,
            })),
        }
    }
    pub fn new_filter(filter: filter::Filter) -> Self {
        Self {
            data: Some(spec::Data::Filter(Filter {
                filter: filter as i32,
            })),
        }
    }

    pub fn watermark(x: u32, y: u32) -> Self {
        Self {
            data: Some(spec::Data::Watermark(Watermark { x, y })),
        }
    }

    pub fn new_blend(blend: blend::Blend) -> Self {
        Self {
            data: Some(spec::Data::Blend(
                Blend {
                    blend: blend as i32,
                }
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Borrow;
    use std::convert::TryInto;

    #[test]
    fn encoded_spec_cloud_be_decoded() {
        let spec1 = Spec::new_resize(600, 600, resize::SampleFilter::CatmullRom);
        let spec2 = Spec::new_filter(filter::Filter::Marine);
        let spec3 = Spec::new_blend(blend::Blend::Xor);
        let spec4 = Spec::watermark(120, 180);
        let image_spec = ImageSpec::new(vec![spec1, spec2, spec3, spec4]);

        let s: String = image_spec.borrow().into();
        println!("{:?}", s);
        assert_eq!(image_spec, s.as_str().try_into().unwrap());
    }
}
