use image::{RgbaImage};
use rand::prelude::*;

const MIN: usize = 1;

pub fn generate(height: u32, width: u32, path: String) -> Result<(), std::io::Error> {
    let imgx = height;
    let imgy = width;

    //https://clrs.cc/
    let colors = vec![[0, 31, 0, 255], [0, 116, 217, 255], [127, 219, 255, 255], [57, 204, 204, 255], [61,153,112, 255], [46,204,64, 255], [255,220,0, 255], [255,133,27, 255], [255,65,54, 255]];

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf: RgbaImage = image::ImageBuffer::new(imgx, imgy);

    // Iterate over the coordinates and pixels of the image
    for (_, _, pixel) in imgbuf.enumerate_pixels_mut() {
        //let r = (0.3 * x as f32) as u8;
        //let b = (0.3 * y as f32) as u8;
        //*pixel = image::Rgb([255, 255, 255]);
        *pixel = image::Rgba([255, 255, 255, 255]);
    }

    let n: usize = thread_rng().gen_range(MIN, 3);

    //horizontal
    for _ in 0..n {
        let x_0: i32 = 0;
        let y_0: i32 = thread_rng().gen_range(0, height as i32);
        let y_1: i32 = y_0;
        let x_1: i32 = width as i32;

        println!("Punkt 1 ({}, {})", x_0, y_0);
        println!("Punkt 2 ({}, {})", x_1, y_1);

        let k: f32 = (y_1 - y_0) as f32 / (x_1 - x_0) as f32; //delta y / delta x
        let d = y_0;

        println!("{} * x + {}", k, d);

        for i in 0..width {
            let y = d as u32;

            let pixel = imgbuf.get_pixel_mut(i, y);
            *pixel = image::Rgba([0, 0, 0, 255]);
        }
    }

    let n: usize = thread_rng().gen_range(MIN, 2);

    //vertical
    for _ in 0..n {
        let x_0: i32 = thread_rng().gen_range(0, width as i32);
        let y_0: i32 = 0;
        let y_1: i32 = height as i32;
        let x_1: i32 = x_0;

        println!("Punkt 1 ({}, {})", x_0, y_0);
        println!("Punkt 2 ({}, {})", x_1, y_1);

        let k: f32 = (y_1 - y_0) as f32 / (x_1 - x_0) as f32; //delta y / delta x
        let d = y_0;

        for i in 0..height {
            let pixel = imgbuf.get_pixel_mut(x_0 as u32, i as u32);
            *pixel = image::Rgba([0, 0, 0, 255]);
        }
    }

    let n: usize = thread_rng().gen_range(1, 10);

    let builder = std::thread::Builder::new()
        .name("reductor".into())
        .stack_size(128 * 1024 * 1024); // 32MB of stack space

    let handler = builder
        .spawn(move || {
            for  _ in 0..n {
                let x = thread_rng().gen_range(0, width);
                let y = thread_rng().gen_range(0, height);
                let color = thread_rng().gen_range(0, colors.len());

                fill(
                    &mut imgbuf,
                    colors[color],
                    x as i32,
                    y as i32,
                    height,
                    width,
                );
            }

            imgbuf.save(path).unwrap();

        })
        .unwrap();

    handler.join().unwrap();
    

    Ok(())
}

fn fill(imgbuf: &mut RgbaImage, color: [u8; 4], x: i32, y: i32, height: u32, width: u32) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }

    let mut pixel = imgbuf.get_pixel_mut(x as u32, y as u32);

    //if *pixel != image::Rgba([0, 0, 0, 255]) && *pixel == image::Rgba([255, 255, 255, 255]) {
    if *pixel == image::Rgba([255, 255, 255, 255]) {
        *pixel = image::Rgba(color);

        fill(imgbuf, color, x, y + 1, height, width);
        fill(imgbuf, color, x - 1, y, height, width);
        fill(imgbuf, color, x, y - 1, height, width);
        fill(imgbuf, color, x + 1, y, height, width);

        //fill(imgbuf, color, x + 1, y, height, width);
        //fill(imgbuf, color, x - 1, y, height, width);

        //fill(imgbuf, color, x, y - 1, height, width);
        //fill(imgbuf, color, x, y + 1, height, width);
    }
}
