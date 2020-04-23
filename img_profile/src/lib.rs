use image::RgbaImage;
use rand::prelude::*;

const MIN: usize = 100;

use std::sync::Mutex;
use std::sync::Arc;

fn change_opacity(x: u8, p: u8) -> u8 {
    255 - ((p as u32 * (255 - x) as u32)  * 100) as u8
}


pub fn generate(height: u32, width: u32, path: String) -> Result<(), std::io::Error> {
    let imgx = height;
    let imgy = width;

    //https://clrs.cc/
    let p: u8 = thread_rng().gen_range(67, 100);

    let colors = vec![
        [0, 31, 0, 255],
        [0, 116, 217, 255],
        [127, 219, 200, 255],
        [57, 204, 204, 255],
        [61, 153, 112, 255],
        [46, 204, 64, 255],
        [255, 220, 0, 255],
        [255, 133, 27, 255],
        [255, 65, 54, 255],
    ]
    .into_iter()
    .map(|color| {
        let [a, b, c, d] = color;
        [change_opacity(a, p), change_opacity(b, p), change_opacity(c, p), d]

    }).collect::<Vec<_>>();

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf: RgbaImage = image::ImageBuffer::new(imgx, imgy);

    // Iterate over the coordinates and pixels of the image
    for (_, _, pixel) in imgbuf.enumerate_pixels_mut() {
        //let r = (0.3 * x as f32) as u8;
        //let b = (0.3 * y as f32) as u8;
        //*pixel = image::Rgb([255, 255, 255]);
        *pixel = image::Rgba([255, 255, 255, 255]);
    }

    let n: usize = thread_rng().gen_range(MIN, 500);

    let mutex = Arc::new(Mutex::new(imgbuf));
    let mut threads = Vec::new();

    for _i in 0..n {
        let x : i32 = thread_rng().gen_range(0, width - 1) as i32;
        let y : i32 = thread_rng().gen_range(0, height - 1) as i32;

        let c = thread_rng().gen_range(0, colors.len());

        let color = colors[c].clone();

        let cpy_width = width.clone();
        let cpy_height = height.clone();
        
        let clone = mutex.clone();
        let handler = std::thread::Builder::new()
            .stack_size(256 * 1024 * 1024)
            .spawn(move || {
            fill(clone, color, x, y, cpy_height, cpy_width);
        }).unwrap();

        threads.push(handler);
    }

    for handler in threads {
        handler.join().expect("Joining failed");
    }

    (mutex.lock().unwrap()).save(path).unwrap();

    Ok(())
}

fn fill(mutex: Arc<Mutex<RgbaImage>>, color: [u8; 4], x: i32, y: i32, height: u32, width: u32) {
    let mut imgbuf = mutex.lock().unwrap();

    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }

    let pixel = imgbuf.get_pixel_mut(x as u32, y as u32);

    //if *pixel != image::Rgba([0, 0, 0, 255]) && *pixel == image::Rgba([255, 255, 255, 255]) {
    if *pixel == image::Rgba([255, 255, 255, 255]) {
        *pixel = image::Rgba(color);

        drop(imgbuf);

        fill(mutex.clone(), color, x, y + 1, height, width);
        fill(mutex.clone(), color, x - 1, y, height, width);
        fill(mutex.clone(), color, x, y - 1, height, width);
        fill(mutex.clone(), color, x + 1, y, height, width);
    }
    else {
        drop(imgbuf);
    }
}
