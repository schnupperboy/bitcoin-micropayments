use std::fs::File;

use qrcode::QrCode;

use image::{
    ImageBuffer,
    DynamicImage,
    ImageFormat,
    Luma
};


const SCALE_FACTOR: u32 = 10;
const BORDER_BITS: u32 = 2;


pub fn create(content: &str) -> Vec<u8> {
	let code = QrCode::new(content.as_bytes()).unwrap();

	let unscaled_img_width = code.width() as u32 + BORDER_BITS;
	let scaled_img_width = SCALE_FACTOR * unscaled_img_width;

	let low_border = 0 as usize;
	let high_border = (unscaled_img_width - 1) as usize;

	// Construct a new by repeated calls to the supplied closure.
	let buf = ImageBuffer::from_fn(scaled_img_width, scaled_img_width, |x, y| {
		let x_bit = (x / SCALE_FACTOR) as usize;
		let y_bit = (y / SCALE_FACTOR) as usize;

		if x_bit == low_border 
				|| y_bit == low_border
				|| x_bit == high_border
				|| y_bit == high_border {
			Luma([255u8])
		} else {
			if code[(x_bit-1, y_bit-1)] {
				Luma([0u8])
			} else {
				Luma([255u8])
			}
		}
	});

	let dynamic_image = DynamicImage::ImageLuma8(buf);

	// for simple debugging
    let mut file_buf = match File::create("/tmp/qr_code.png") {
        Result::Ok(f) => f,
        Result::Err(err) =>
          panic!("Error opening file: {:?}", err),
    };
   
	let _ = dynamic_image.save(&mut file_buf, ImageFormat::PNG);

	let mut mem_buf: Vec<u8> = Vec::new();
	let _ = dynamic_image.save(&mut mem_buf, ImageFormat::PNG);

	mem_buf
}