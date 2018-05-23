use common::{DisposalMethod, Frame};
use encoder::Encoder;
use reader::ColorOutput;
use reader::Decoder;
use reader::StreamingDecoder;
use std::cell::Cell;
use std::io;
use std::io::prelude::*;
use std::sync::Arc;

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
    count: u16,
    duration: u16,
    is_loop: bool,
    // ext buffer
    // ext: (u8, Vec<u8>, bool),
    Frames: Vec<Arc<Frame<'static>>>,
}

impl<R: Read + Copy> BatchGif<R> {
    /// new create BatchGif
    pub fn new(r: R) -> Result<BatchGif<R>, ()> {
        match Decoder::new(r).read_info() {
            Ok(mut decode) => {
                //println!("decode.is_loop() {}", decode.is_loop());
                let mut bgif = BatchGif {
                    r: r,
                    width: decode.width(),
                    height: decode.height(),
                    is_loop: false,
                    background_color_index: decode.bg_color(),
                    global_palette: decode.global_palette().unwrap().to_vec(),
                    Frames: Vec::new(),
                    count: 0,
                    duration: 0,
                };
                while let Some(frame) = decode.read_next_frame().unwrap() {
                    bgif.count += 1;
                    bgif.duration += frame.delay;
                    bgif.Frames.push(Arc::new(frame.clone()));
                    // Process every frame
                }
                bgif.is_loop = decode.is_loop();
                Ok(bgif)
            }
            Err(_) => return Err(()),
        }
    }

    /// get_gif_by_index get the gif of the frame by index
    pub fn get_gif_by_index(&self, index: usize) -> Vec<u8> {
        assert!(index <= self.Frames.len() - 1);
        let mut image = Vec::new();
        let frame = &self.Frames[index];
        {
            let mut encode;
            let img = &mut image;
            match frame.palette {
                Some(_) => {
                    encode = Encoder::new(img, self.width, self.height, &[], false).unwrap();
                }
                None => {
                    encode = Encoder::new(
                        img,
                        self.width,
                        self.height,
                        self.global_palette.as_slice(),
                        false,
                    ).unwrap();
                }
            }
            encode.write_frame(frame).unwrap();
        }
        image
    }

    /// get_gif_by_index get the gif of the frame by index
    pub fn optimize_gif(&self) -> Vec<u8> {
        let mut image = Vec::new();
        {
            let mut img = &mut image;
            let mut encode = Encoder::new(
                img,
                self.width,
                self.height,
                self.global_palette.as_slice(),
                self.is_loop,
            ).unwrap();
            let is_optimize = if self.duration / self.count >= 20 {
                false
            } else {
                true
            };
            let mut tmp_count: usize = 0;
            let mut tmp_duration: u16 = 0;
            let mut tmp_delay: u16 = 0;
            let mut total_count: u16 = 0;
            println!("frame count: {}", self.count);
            println!("frame count: {}", self.Frames.len());
            for mut frame in self.Frames.clone() {
                total_count += 1;
                if is_optimize {
                    tmp_duration += frame.delay;
                    tmp_count += 1;
                    if tmp_duration > 20 && tmp_count > 5 {
                        println!("remove frame count: {}", total_count);
                        tmp_duration = 0;
                        tmp_count = 0;
                        tmp_delay = frame.delay;
                        continue;
                    }
                }
                if tmp_delay != 0 {
                    frame = Arc::new(Frame {
                        delay: (frame.delay + tmp_delay) / 2,
                        dispose: DisposalMethod::Previous,
                        transparent: frame.transparent,
                        needs_user_input: frame.needs_user_input,
                        top: frame.top,
                        left: frame.left,
                        width: frame.width,
                        height: frame.height,
                        interlaced: frame.interlaced,
                        palette: frame.palette.clone(),
                        buffer: frame.buffer.clone(),
                    });
                    tmp_delay = 0;
                }
                println!("delaym {:?}", frame.dispose);
                encode.write_frame(Arc::make_mut(&mut frame)).unwrap();
            }
        }
        image
    }

    /// is_loop of the image
    pub fn is_loop(&self) -> bool {
        self.is_loop
    }
}
