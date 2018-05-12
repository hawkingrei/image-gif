use common::Frame;
use encoder::Encoder;
use reader::ColorOutput;
use reader::Decoder;
use reader::StreamingDecoder;
use std::io;
use std::io::prelude::*;

///  new gif process
pub struct BatchGif<R: Read + Copy> {
    r: R,
    //decoder: Decoder<R>,
    width: u16,
    height: u16,
    //global_color_table: Vec<u8>,
    //background_color: [u8; 4],
    background_color_index: Option<usize>,
    global_palette: Vec<u8>,
    // ext buffer
    // ext: (u8, Vec<u8>, bool),
    Frames: Vec<Frame<'static>>,
}

impl<R: Read + Copy> BatchGif<R> {
    /// new create BatchGif
    pub fn new(r: R) -> Result<BatchGif<R>, ()> {
        match Decoder::new(r).read_info() {
            Ok(mut decode) => {
                let mut bgif = BatchGif {
                    r: r,
                    width: decode.width(),
                    height: decode.height(),
                    background_color_index: decode.bg_color(),
                    global_palette: decode.global_palette().unwrap().to_vec(),
                    Frames: Vec::new(),
                };
                while let Some(frame) = decode.read_next_frame().unwrap() {
                    bgif.Frames.push(frame.clone());
                    // Process every frame
                }
                Ok(bgif)
            }
            Err(_) => return Err(()),
        }
    }

    /// get_gif_by_index get the gif of the frame by index
    pub fn get_gif_by_index(&self, index: usize) -> Vec<u8> {
        assert!(index <= self.Frames.len() - 1);

        let mut encode;
        let frame = &self.Frames[index];
        let mut image = Vec::new();
        {
            let mut img = image.clone();
            match frame.palette {
                Some(_) => {
                    encode = Encoder::new(img, self.width, self.height, &[]).unwrap();
                }
                None => {
                    encode =
                        Encoder::new(img, self.width, self.height, self.global_palette.as_slice())
                            .unwrap();
                }
            }
            encode.write_frame(frame).unwrap();
        }
        return image;
    }
}
