use image::RgbaImage;
use rand::prelude::*;

const MIN: usize = 100;
const RECT_HEIGHT: u32 = 50;
const RECT_WIDTH: u32 = 50;

use std::sync::Mutex;
use std::sync::Arc;

fn get_rect(x: u32, y: u32, height: u32, width: u32) -> Vec<(u32, u32)> {
    let mut points = Vec::with_capacity((RECT_HEIGHT * RECT_WIDTH) as usize);

    for i in x..(x + RECT_WIDTH) {
        for j in y..(y + RECT_HEIGHT) {
            points.push((i,j));
        }
    }

    points.into_iter().filter(|(x, y)| (x < &width && x >= &0) && (y >= &0 && y < &height)).collect()
}

fn change_opacity(x: u8, p: u8) -> u8 {
    255 - ((p as u32 * (255 - x) as u32)  * 100) as u8
}

fn get_distance(p: (u32, u32), center: (u32, u32)) -> u32 {
    let w = (p.0 as i32 - center.0 as i32, p.1 as i32 - center.1 as i32);

    ((w.0.pow(2) + w.1.pow(2)) as f32).sqrt().round() as u32
}


pub fn generate(height: u32, width: u32, path: String) -> Result<(), std::io::Error> {
    let imgx = height;
    let imgy = width;

    let center_x = width / 2;
    let center_y = width / 2;

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

    for i in 0..n {
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
        handler.join();
    }

    (mutex.lock().unwrap()).save(path).unwrap();

    Ok(())
}

fn fill(mutex: Arc<Mutex<RgbaImage>>, color: [u8; 4], x: i32, y: i32, height: u32, width: u32) {
    let mut imgbuf = mutex.lock().unwrap();

    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }

    let mut pixel = imgbuf.get_pixel_mut(x as u32, y as u32);

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
