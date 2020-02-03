use rand::prelude::*;

const MIN: usize = 5;

pub fn generate(height: u32, width: u32, path: &str) -> Result<(), std::io::Error>{
    let imgx = height;
    let imgy = width;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    // Iterate over the coordinates and pixels of the image
    for (_, _, pixel) in imgbuf.enumerate_pixels_mut() {
        //let r = (0.3 * x as f32) as u8;
        //let b = (0.3 * y as f32) as u8;
        *pixel = image::Rgb([255, 255, 255]);
    }

    let n: usize = thread_rng().gen_range(MIN, 12);

    for _ in 0..n {
        let x: u32 = thread_rng().gen_range(0, width);
        let y: u32 = thread_rng().gen_range(0, height);

        let pixel = imgbuf.get_pixel_mut(x, y);
        *pixel = image::Rgb([0, 0, 0]);

        let step = 40;

        if x + step >= imgx {
            continue;
        }

        if y + step >= imgy {
            continue;
        }

        for i in x..(x + step) {
            if i < imgx {
                for j in y..(y + step) {
                    if j < imgy {
                        let pixel = imgbuf.get_pixel_mut(i, j);
                        *pixel = image::Rgb([0, 0, 0]);
                    }
                }
            }
        }
    }

    imgbuf.save(path)
}
