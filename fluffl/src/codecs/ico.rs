#[allow(unused_imports)]
use std::{
    fmt::Debug,
    io::{self, Cursor, Read, Seek, SeekFrom, Write},
    mem,
};

#[derive(Debug)]
pub struct DIBHeader {
    pub header_size: u32,
    pub width: u32,
    pub height: u32,
    pub color_planes: u16,
    pub bpp: u16,
    pub compression: u32,
    pub size: u32,
    pub res_x: u32,
    pub res_y: u32,
    pub pallete: u32,
    pub important_colors: u32,
}
impl DIBHeader {
    pub fn load<T>(input: &mut T) -> io::Result<Self>
    where
        T: Read,
    {
        let mut scratch_space = [0u8; 256];
        Ok(Self {
            header_size: read_primitive(input, &mut scratch_space)?,
            width: read_primitive(input, &mut scratch_space)?,
            height: read_primitive(input, &mut scratch_space)?,
            color_planes: read_primitive(input, &mut scratch_space)?,
            bpp: read_primitive(input, &mut scratch_space)?,
            compression: read_primitive(input, &mut scratch_space)?,
            size: read_primitive(input, &mut scratch_space)?,
            res_x: read_primitive(input, &mut scratch_space)?,
            res_y: read_primitive(input, &mut scratch_space)?,
            pallete: read_primitive(input, &mut scratch_space)?,
            important_colors: read_primitive(input, &mut scratch_space)?,
        })
    }
}

pub struct IcoEntry {
    pub width: u8,
    pub height: u8,
    pub colors: u8,
    pub reserved: u8,
    pub planes: u16,
    pub bits_per_pixel: u16,
    pub bitmap_filesize: u32,
    pub offset: u32,
    pub bitmap: Vec<u8>,
}
impl IcoEntry {
    pub fn load<T>(input: &mut T) -> io::Result<Self>
    where
        T: Read,
    {
        let mut scratch_space = [0u8; 256];
        Ok(Self {
            width: read_primitive(input, &mut scratch_space)?,
            height: read_primitive(input, &mut scratch_space)?,
            colors: read_primitive(input, &mut scratch_space)?,
            reserved: read_primitive(input, &mut scratch_space)?,
            planes: read_primitive(input, &mut scratch_space)?,
            bits_per_pixel: read_primitive(input, &mut scratch_space)?,
            bitmap_filesize: read_primitive(input, &mut scratch_space)?,
            offset: read_primitive(input, &mut scratch_space)?,
            bitmap: vec![],
        })
    }
}

impl Debug for IcoEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "width = {:?} ", self.width)?;
        writeln!(f, "height = {:?} ", self.height)?;
        writeln!(f, "colors = {:?} ", self.colors)?;
        writeln!(f, "reserved = {:?} ", self.reserved)?;
        writeln!(f, "planes = {:?} ", self.planes)?;
        writeln!(f, "bits_per_pixel = {:?} ", self.bits_per_pixel)?;
        writeln!(f, "num_bytes = {:?} ", self.bitmap_filesize)?;
        writeln!(f, "offset = {:?} ", self.offset)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct IcoHeader {
    pub reserved_0: u16,
    pub img_type: u16,
    pub num_images: u16,
}

impl IcoHeader {
    pub fn load<T>(input: &mut T) -> io::Result<Self>
    where
        T: Read,
    {
        let mut scratch_space = [0u8; 256];
        Ok(Self {
            reserved_0: read_primitive(input, &mut scratch_space)?,
            img_type: read_primitive(input, &mut scratch_space)?,
            num_images: read_primitive(input, &mut scratch_space)?,
        })
    }
}
/// A microsoft icon
/// used file spec on wiki here: https://en.wikipedia.org/wiki/ICO_(file_format)#Outline
#[derive(Debug)]
pub struct Ico {
    pub entries: Vec<IcoEntry>,
}
impl Ico {
    pub fn load<T>(mut input: T) -> io::Result<Self>
    where
        T: Read + Seek,
    {
        let header = IcoHeader::load(&mut input)?;

        println!("ico header = {:?}", header);

        let mut entries = Vec::with_capacity(header.num_images as usize);
        for _ in 0..header.num_images {
            entries.push(IcoEntry::load(&mut input)?);
        }
        let mut bmp_bytes = vec![0u8; 512 * 512 * 4];

        for entry in entries.iter_mut() {
            let bitmap_filesize = entry.bitmap_filesize as usize;
            let offset = entry.offset as u64;
            let width = entry.width as usize;
            let height = entry.height as usize;

            // make sure vector is same size as bitmap
            bmp_bytes.resize(bitmap_filesize, 0);

            // seek to the start of the bitmap data
            input.seek(SeekFrom::Start(offset))?;

            // read the bitmap data into the vector
            input.read_exact(&mut bmp_bytes)?;

            // according to the spec, the BMP file will have NO file-header (DIB header is still included)
            // Update: turns out this was all pointless, but im keeping here as reference
            // let mut bmp_file = Cursor::new(&bmp_bytes[..]);
            // let bmp_header = DIBHeader::load(&mut bmp_file)?;
            // println!("bmp header = {:?}", bmp_header);

            let padding = 0;
            let bytes_per_pixel = entry.bits_per_pixel as usize / 8;
            let image_row_bytes = (width * bytes_per_pixel) + padding;
            let pixel_info = &bmp_bytes[40..];

            // bitmaps are stored flipped for some dumb reason
            // so I have to unflip them
            for i in 0..height {
                for j in 0..width {
                    let ofx = ((height - i - 1) * image_row_bytes) + bytes_per_pixel * j;
                    for channel in 0..bytes_per_pixel {
                        entry.bitmap.push(pixel_info[ofx + channel]);
                    }
                }
            }
        }

        Ok(Self { entries })
    }

    pub fn dump_ppm<P: AsRef<std::path::Path>>(&self, path: P) -> io::Result<()> {
        let mut ppm = std::fs::File::create(path)?;
        writeln!(ppm, "P3")?;
        writeln!(ppm, "{} {}", self.entries[0].width, self.entries[0].height)?;
        writeln!(ppm, "{max_color_val}", max_color_val = 255)?;
        for pixel in self.entries[0].bitmap.chunks_exact(4) {
            writeln!(ppm, " {} {} {} ", pixel[2], pixel[1], pixel[0]).unwrap();
        }
        Ok(())
    }
}

pub fn read_primitive<T: Read, Output: Copy + Default>(
    mem: &mut T,
    scratch_space: &mut [u8],
) -> io::Result<Output> {
    let primitive_bytes = &mut scratch_space[0..mem::size_of::<Output>()];
    mem.read_exact(primitive_bytes)?;
    let primitive_raw_ptr = primitive_bytes.as_ptr() as *const Output;
    Ok(unsafe { *primitive_raw_ptr })
}

#[test]
pub fn parse_ico() {
    // let cwd = std::env::current_dir();
    // println!("my cwd is = {:?}", cwd);
    let ico_file = std::fs::File::open("../resources/test.ico").unwrap();
    let ico = Ico::load(ico_file).unwrap();
    ico.dump_ppm("../dump/ico.ppm").unwrap();

    let ico_file = std::fs::File::open("../resources/pokeball.ico").unwrap();
    let ico = Ico::load(ico_file).unwrap();
    ico.dump_ppm("../dump/pokeball.ppm").unwrap();
}
