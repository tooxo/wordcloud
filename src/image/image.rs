use edge_detection::Detection;

use crate::types::rect::Rect;
use image::{DynamicImage, GenericImageView, GrayImage, Rgba};

pub(crate) type Dimensions = Rect<usize>;

pub(crate) fn canny_algorithm(image: &GrayImage, sigma: f32) -> Detection {
    let det = edge_detection::canny(image.clone(), sigma, 0.3, 0.05);

    det.as_image().save("canny.png").expect("save failed");

    det
}

pub(crate) fn average_color_for_rect(image: &DynamicImage, rect: &Rect<u32>) -> Rgba<u8> {
    assert!(image.width() >= rect.max.x && image.height() >= rect.max.y);
    let colors: Vec<[u8; 4]> = (rect.min.x..rect.max.x)
        .flat_map(|x| {
            (rect.min.y..rect.max.y)
                .map(|y| image.get_pixel(x, y).0)
                .collect::<Vec<[u8; 4]>>()
        })
        .collect();

    if !colors.is_empty() {
        let summed_red: usize =
            (colors.iter().map(|c| c[0] as usize).sum::<usize>()) / colors.len();
        let summed_green: usize =
            colors.iter().map(|c| c[1] as usize).sum::<usize>() / colors.len();
        let summed_blue: usize = colors.iter().map(|c| c[2] as usize).sum::<usize>() / colors.len();
        let summed_alpha: usize =
            colors.iter().map(|c| c[3] as usize).sum::<usize>() / colors.len();

        Rgba([
            summed_red as u8,
            summed_green as u8,
            summed_blue as u8,
            summed_alpha as u8,
        ])
    } else {
        Rgba([0, 0, 0, 0])
    }
}

pub(crate) fn color_to_rgb_string(rgba: &Rgba<u8>) -> String {
    format!("rgb({}, {}, {})", rgba.0[0], rgba.0[1], rgba.0[2], )
}
