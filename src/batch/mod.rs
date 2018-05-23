use common::{DisposalMethod, Frame};
use encoder::Encoder;
use reader::ColorOutput;
use reader::Decoder;
use reader::StreamingDecoder;
use std::cell::Cell;
use std::io;
use std::io::prelude::*;
use std::sync::Arc;

struct BatchFrame {
    Frame: Frame<'static>,
    transparent_color: Option<[u8; 3]>,
    color_table: Vec<u8>,
}

impl BatchFrame {
    pub fn new(frame: Frame<'static>, global_palette: Option<&[u8]>) -> BatchFrame {
        let mut fframe = frame.clone();
        let mut color_table;
        let tcolor;
        {
            color_table = match fframe.palette {
                Some(p) => p,
                None => global_palette.unwrap().to_vec(),
            };
            tcolor = match fframe.transparent {
                Some(p) => {
                    let mut count = 0;
                    let mut result = Some([0; 3]);
                    for color in color_table.chunks_mut(3) {
                        if count + 1 == p {
                            result = Some([color[0], color[1], color[2]]);
                        }
                        count = count + 1;
                    }
                    result
                }
                None => None,
            };
        }

        BatchFrame {
            Frame: frame,
            color_table: color_table,
            transparent_color: tcolor,
        }
    }
}

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
    only_global_color: bool,

    Frames: Vec<Arc<BatchFrame>>,
}

impl<R: Read + Copy> BatchGif<R> {
    /// new create BatchGif
    pub fn new(r: R) -> Result<BatchGif<R>, ()> {
        match Decoder::new(r).read_info() {
            Ok(mut decode) => {
                //println!("decode.is_loop() {}", decode.is_loop());
                let mut bgif = BatchGif {
                    r: r,
                    width: decode.width().clone(),
                    height: decode.height(),
                    is_loop: false,
                    background_color_index: decode.bg_color(),
                    global_palette: decode.global_palette().unwrap().to_vec(),
                    Frames: Vec::new(),
                    count: 0,
                    duration: 0,
                    only_global_color: true,
                };

                println!("{:?}", bgif.global_palette);
                {
                    while let Some(frame) = decode.read_next_frame().unwrap() {
                        bgif.count += 1;
                        bgif.duration += frame.delay;
                        bgif.Frames.push(Arc::new(BatchFrame::new(
                            frame.clone(),
                            Some(&bgif.global_palette),
                        )));
                        match frame.palette {
                            Some(_) => bgif.only_global_color = false,
                            None => {}
                        }
                        // Process every frame
                    }
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
            match frame.Frame.palette {
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
            encode.write_frame(&frame.Frame).unwrap();
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
                    tmp_duration += frame.Frame.delay;
                    tmp_count += 1;
                    if tmp_duration > 20 && tmp_count > 5 {
                        println!("remove frame count: {}", total_count);
                        tmp_duration = 0;
                        tmp_count = 0;
                        tmp_delay = frame.Frame.delay;
                        continue;
                    }
                }
                if tmp_delay != 0 {
                    frame = Arc::new(BatchFrame::new(
                        Frame {
                            delay: (frame.Frame.delay + tmp_delay) / 2,
                            dispose: DisposalMethod::Previous,
                            transparent: frame.Frame.transparent,
                            needs_user_input: frame.Frame.needs_user_input,
                            top: frame.Frame.top,
                            left: frame.Frame.left,
                            width: frame.Frame.width,
                            height: frame.Frame.height,
                            interlaced: frame.Frame.interlaced,
                            palette: frame.Frame.palette.clone(),
                            buffer: frame.Frame.buffer.clone(),
                        },
                        Some(&self.global_palette),
                    ));
                    tmp_delay = 0;
                }
                println!("delaym {:?}", frame.Frame.dispose);
                encode.write_frame(&frame.Frame).unwrap();
            }
        }
        image
    }

    /// is_loop of the image
    pub fn is_loop(&self) -> bool {
        self.is_loop
    }
}
