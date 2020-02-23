use image::RgbaImage;
use rand::prelude::*;

const MIN: usize = 50;
const RECT_HEIGHT: u32 = 50;
const RECT_WIDTH: u32 = 50;

fn get_rect(x: u32, y: u32, height: u32, width: u32) -> Vec<(u32, u32)> {
    let mut points = Vec::with_capacity((RECT_HEIGHT * RECT_WIDTH) as usize);
    /*
    points.push((x, y));
    
    //up
    points.extend((y..(y + RECT_HEIGHT / 2)).map(|w| (x, w)));
    
    //down
    points.extend((y..(y - RECT_HEIGHT / 2)).map(|w| (x, w)));

    //left
    points.extend((x..(x - RECT_WIDTH / 2)).map(|w| (w, y)));

    //right
    points.extend((x..(x + RECT_WIDTH / 2)).map(|w| (w, y)));
    */

    for i in x..(x + RECT_WIDTH) {
        for j in y..(y + RECT_HEIGHT) {
            points.push((i,j));
        }
    }

    //points.into_iter().filter(|(x, y)| (x >= &width || x < &0) || (y < &0 || y >= &height)).collect()
    points.into_iter().filter(|(x, y)| (x < &width && x >= &0) && (y >= &0 && y < &height)).collect()
}

fn change_opacity(x: u8, P: u8) -> u8 {
    //255 - P*(255-X)
    255 - ((P as u32 * (255 - x) as u32)  * 100) as u8
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

    let n: usize = thread_rng().gen_range(MIN, 130);

    for i in 0..n {
        let x = thread_rng().gen_range(0, width - 1);
        let y = thread_rng().gen_range(0, height - 1);

        let rect = get_rect(x, y, height, width);
        let c = thread_rng().gen_range(0, colors.len());
        let color = colors[c];

        for (x, y) in rect {
            let mut pixel = imgbuf.get_pixel_mut(x, y);

            *pixel = image::Rgba(color);
        }
    }

    imgbuf.save(path).unwrap();

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
