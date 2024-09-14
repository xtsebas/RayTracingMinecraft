use std::fs::File;
use std::io::{Write, BufWriter};

const BMP_HEADER_SIZE: usize = 54;
const BMP_PIXEL_OFFSET: usize = 54;
const BMP_BITS_PER_PIXEL: usize = 24;

pub fn write_bmp_file(
    file_path: &str,
    buffer: &[u32],
    width: usize,
    height: usize,
) {
    //TODO: create a buffered writer for the file
    let file = File::create(file_path).unwrap();
    let mut writer = BufWriter::new(file);

    //wrute the BMP header
    write_bmp_header(&mut writer, width, height);

    // write the pixel data from the framebuffer
    write_pixel_data(&mut writer, buffer, width, height);

}

fn write_bmp_header(
    file: &mut BufWriter<File>,
    width: usize,
    height: usize,
)  {
    //TODO: calculate file size and pixel data size
    let file_size = (BMP_HEADER_SIZE + width * height * 3) as u32;
    let reserved: u32 = 0;
    let offset = BMP_PIXEL_OFFSET as u32;
    let dib_header_size: u32 = 40;
    let planes: u16 = 1;
    let bits_per_pixel: u16 = BMP_BITS_PER_PIXEL as u16;
    let compression: u32 = 0;
    let image_size: u32 = (width * height * 3) as u32;
    let x_ppm: u32 = 0; // 72 DPI
    let y_ppm: u32 = 0; // 72 DPI
    let total_colors: u32 = 0;
    let important_colors: u32 = 0;

    //write bmp signature
    file.write_all(b"BM").unwrap();

    //write file size, reserved bytes, and pixel offset
    file.write_all(&file_size.to_le_bytes()).unwrap();
    file.write_all(&reserved.to_le_bytes()).unwrap();
    file.write_all(&offset.to_le_bytes()).unwrap();

    //write header size, image width, and image height
    file.write_all(&dib_header_size.to_le_bytes()).unwrap();
    file.write_all(&(width as u32).to_le_bytes()).unwrap();
    file.write_all(&(height as u32).to_le_bytes()).unwrap();


    //write color planes and bits per pixel
    file.write_all(&planes.to_le_bytes()).unwrap();
    file.write_all(&bits_per_pixel.to_le_bytes()).unwrap();

    //wrtie compression method, pixel data size, and resolution
    file.write_all(&compression.to_le_bytes()).unwrap();
    file.write_all(&image_size.to_le_bytes()).unwrap();
    file.write_all(&x_ppm.to_le_bytes()).unwrap();
    file.write_all(&y_ppm.to_le_bytes()).unwrap();

    //write number of colors and important colors
    file.write_all(&total_colors.to_le_bytes()).unwrap();
    file.write_all(&important_colors.to_le_bytes()).unwrap();

}

fn write_pixel_data(
    file: &mut BufWriter<File>,
    buffer: &[u32],
    width: usize,
    height: usize,
) {
    // Calcular el tamaño del padding para cada fila

    for y in (0..height).rev() {
        for x in 0..width {
            let index = y * width + x;
            let color = buffer[index];
            let r = ((color >> 16) & 0xFF) as u8;
            let g = ((color >> 8) & 0xFF) as u8;
            let b = (color & 0xFF) as u8;
            file.write_all(&[b, g, r]).unwrap();
        }
    }

}