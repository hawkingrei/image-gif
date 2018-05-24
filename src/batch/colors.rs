
#[derive(Debug,Copy,Clone)]
struct color {
     haspixel :u8,      /* semantics assigned by user */
     gfc_red :u8,       /* red component (0-255) */
     gfc_green :u8,     /* green component (0-255) */
     gfc_blue :u8,      /* blue component (0-255) */
     pixel :u32,        /* semantics assigned by user */
}


#[derive(Debug,Copy,Clone)]
struct color_map {
    colors Vec<color>,
}

