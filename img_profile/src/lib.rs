#![recursion_limit = "2048"]

use rand::prelude::*;
use image::{GenericImage, GenericImageView, ImageBuffer, RgbImage};

const MIN: usize = 2;

pub fn generate(height: u32, width: u32, path: &str) -> Result<(), std::io::Error>{
    let imgx = height;
    let imgy = width;

    //https://clrs.cc/
    let colors = vec![
        [255, 255, 255],
        [0, 31, 0],
        [0,116,217],
        [127,219,255],
        [57,204,204]
    ];

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf : RgbImage = image::ImageBuffer::new(imgx, imgy);

    // Iterate over the coordinates and pixels of the image
    for (_, _, pixel) in imgbuf.enumerate_pixels_mut() {
        //let r = (0.3 * x as f32) as u8;
        //let b = (0.3 * y as f32) as u8;
        //*pixel = image::Rgb([255, 255, 255]);
        *pixel = image::Rgb([255, 255, 255]);
    }

    let n: usize = thread_rng().gen_range(MIN, 5);

    for _ in 0..n {
        let x_0: i32 = 0;
        let y_0: i32 = thread_rng().gen_range(0, height as i32);
        let y_1: i32 = thread_rng().gen_range(0, height as i32);
        let x_1: i32 = width as i32;

        println!("Punkt 1 ({}, {})", x_0, y_0);
        println!("Punkt 2 ({}, {})", x_1, y_1);

        let k : f32 = ((y_1 - y_0) as f32 / (x_1 - x_0) as f32 ) ; //delta y / delta x
        let d = y_0 ;

        println!("{} * x + {}", k, d);

        for i in 0..width {
            let y = (k * (i as f32) + d as f32) as u32;

            let pixel = imgbuf.get_pixel_mut(i, y);
            *pixel = image::Rgb([0, 0, 0]);

            /*
            //THICKER LINES
            if y > 0 {
                let pixel = imgbuf.get_pixel_mut(i, y - 1);
                *pixel = image::Rgb([0, 0, 0]);
            }

            if y < height {
                let pixel = imgbuf.get_pixel_mut(i, y + 1);
                *pixel = image::Rgb([0, 0, 0]);
            }
            */
        }

        fill(&mut imgbuf, colors[1], 0, 0, height, width);

        /*
        for i in 0..width {
            for z in 0..height {
                let mut pixel = imgbuf.get_pixel_mut(i, z);

                if *pixel != image::Rgb([0, 0, 0]) {
                    
                }
            }

        }
        */

    }

    imgbuf.save(path)
}

fn fill(imgbuf: &mut RgbImage, color: [u8; 3], x: i32, y: i32, height: u32, width: u32) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }

    let mut pixel = imgbuf.get_pixel_mut(x as u32, y as u32);

    if *pixel != image::Rgb([0, 0, 0]) && *pixel == image::Rgb([255, 255, 255]) {
        *pixel = image::Rgb(color);

        fill(imgbuf, color, x, y + 1, height, width);
        fill(imgbuf, color, x - 1, y, height, width);
        fill(imgbuf, color, x, y - 1, height, width);
        fill(imgbuf, color, x + 1, y, height, width);

        //fill(imgbuf, color, x + 1, y, height, width);
        //fill(imgbuf, color, x - 1, y, height, width);

        //fill(imgbuf, color, x, y - 1, height, width);
        //fill(imgbuf, color, x, y + 1, height, width);
    }
    else if *pixel == image::Rgb([0, 0, 0]) {
        //https://clrs.cc/
        let colors = vec![
            [0, 31, 0],
            [0,116,217],
            [127,219,255],
            [57,204,204]
        ];

        let c = thread_rng().gen_range(0, colors.len());

        fill(imgbuf, colors[c], x + 1, y, height, width);
        fill(imgbuf, colors[c], x, y + 1, height, width);
    }
}
