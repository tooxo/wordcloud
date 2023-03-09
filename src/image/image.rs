use image::{DynamicImage, GrayAlphaImage, GrayImage, ImageBuffer, Luma, LumaA, Rgb, RgbaImage, RgbImage};
use image::buffer::Pixels;
use image::imageops::{grayscale, grayscale_alpha};
use rayon::prelude::{IntoParallelIterator, ParallelBridge};

pub(crate) fn convert_image_to_greyscale(image: &DynamicImage) -> GrayImage {
    grayscale(image)
}

pub(crate) fn canny_algorithm(image: &GrayImage, sigma: f32) {
    let det = edge_detection::canny(
        image.clone(), sigma,
        0.3, 0.05,
    );

    det.as_image().save("canny.png").expect("save failed");
}