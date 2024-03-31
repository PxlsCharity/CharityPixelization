use charity_pixelization::{process_image, Color, I2PState, Sprite};
use image::{GenericImageView, ImageBuffer, Rgba};
use stopwatch::Stopwatch;

fn main() {
    let palette = [
        "#6D001AFF",
        "#BE0039FF",
        "#FF4500FF",
        "#FFA800FF",
        "#FFD635FF",
        "#FFF8B8FF",
        "#00A368FF",
        "#00CC78FF",
        "#7EED56FF",
        "#00756FFF",
        "#009EAAFF",
        "#00CCC0FF",
        "#2450A4FF",
        "#3690EAFF",
        "#51E9F4FF",
        "#493AC1FF",
        "#6A5CFFFF",
        "#94B3FFFF",
        "#811E9FFF",
        "#B44AC0FF",
        "#E4ABFFFF",
        "#DE107FFF",
        "#FF3881FF",
        "#FF99AAFF",
        "#6D482FFF",
        "#9C6926FF",
        "#FFB470FF",
        "#000000FF",
        "#515252FF",
        "#898D90FF",
        "#D4D7D9FF",
        "#FFFFFFFF",
    ];
    let mut state = I2PState {
        pixel_distance_mode: charity_pixelization::DistanceMode::Redmean,
        palette: palette.iter().map(|c| c.parse().unwrap()).collect(),
        ..Default::default()
    };
    let image = image::open("lenna.png").unwrap();
    let mut output = Sprite {
        width: image.width() as usize,
        height: image.height() as usize,
        data: vec![Default::default(); image.width() as usize * image.height() as usize],
    };
    for (x, y, pixel) in image.pixels() {
        let pixel = pixel.0;
        output.set_pixel(
            x as usize,
            y as usize,
            Color::new(pixel[0], pixel[1], pixel[2], pixel[3]),
        );
    }
    let input = output.clone();
    let stopwatch = Stopwatch::start_new();
    process_image(&mut state, &input, &mut output);
    println!("processing took {:?}", stopwatch.elapsed());

    let mut imgbuf: ImageBuffer<Rgba<u8>, Vec<_>> =
        ImageBuffer::new(output.width as u32, output.height as u32);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let sprite_pixel = output.get_pixel(x as usize, y as usize).unwrap();
        *pixel = Rgba([
            sprite_pixel.red,
            sprite_pixel.green,
            sprite_pixel.blue,
            sprite_pixel.alpha,
        ]);
    }

    imgbuf.save("output.png").unwrap();
}
