use std::fs::File;

use qrcode::QrCode;

use image::{
    ImageBuffer,
    DynamicImage,
    ImageFormat,
    Luma
};

const SCALE_FACTOR: u32 = 10;


pub fn create(content: &str) -> Vec<u8> {
	let code = QrCode::new(content.as_bytes()).unwrap();

	let img_width = (code.width() as u32) * SCALE_FACTOR;

	// Construct a new by repeated calls to the supplied closure.
	let buf = ImageBuffer::from_fn(img_width, img_width, |x, y| {
		let x_bit = (x / SCALE_FACTOR) as usize;
		let y_bit = (y / SCALE_FACTOR) as usize;

		if code[(x_bit, y_bit)] {
			Luma([255u8])
		} else {
			Luma([0u8])
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