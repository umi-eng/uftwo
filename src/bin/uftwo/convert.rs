use anyhow::Error;
use clap::Parser;
use std::{
    ffi::OsStr,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};
use uftwo::Block;
use zerocopy::AsBytes;

#[derive(Parser)]
pub struct Cmd {
    #[arg(value_name = "INPUT")]
    input_path: PathBuf,
    #[arg(value_name = "OUTPUT")]
    output_path: Option<PathBuf>,
    #[clap(long, default_value_t = 0x2000)]
    target_addr: usize,
}

impl Cmd {
    pub fn run(self) -> anyhow::Result<()> {
        let extension = match self.input_path.extension() {
            Some(ext) => ext,
            None => {
                return Err(Error::msg("failed"));
            }
        };

        let input_uf2 =
            extension == OsStr::new("uf2") || extension == OsStr::new("UF2");

        let output_path = if let Some(path) = self.output_path {
            path
        } else {
            let mut path = self.input_path.clone();

            if !input_uf2 {
                // add extension
                path.set_extension("uf2");
            } else {
                path.set_extension("bin");
            }

            path
        };

        println!("Converting {:?} to {:?}", self.input_path, output_path);

        if input_uf2 {
            uf2_to_bin(self.input_path, output_path)
        } else {
            bin_to_uf2(self.input_path, output_path, self.target_addr)
        }
    }
}

/// Binary to UF2.
fn bin_to_uf2(
    input: PathBuf,
    output: PathBuf,
    target_addr: usize,
) -> anyhow::Result<()> {
    let mut input_file = File::open(input)?;
    let mut output_file = File::create(output)?;

    let mut binary = Vec::new();
    input_file.read_to_end(&mut binary)?;

    let total_blocks = binary.chunks(256).count();

    binary.chunks(256).enumerate().for_each(|(index, chunk)| {
        let mut block = Block::default();

        block.payload_size = chunk.len() as u32;
        block.target_addr = target_addr as u32;

        block.block_number = index as u32;
        block.total_block = total_blocks as u32;

        block.data[0..chunk.len()].copy_from_slice(chunk);

        output_file.write(block.as_bytes()).unwrap();
    });

    println!(
        "Written {} bytes into {} blocks.",
        binary.len(),
        total_blocks
    );

    output_file.flush()?;

    Ok(())
}

/// UF2 to binary.
fn uf2_to_bin(input: PathBuf, output: PathBuf) -> anyhow::Result<()> {
    let mut input_file = File::open(input)?;
    let mut output_file = File::create(output)?;

    let mut binary: Vec<u8> = vec![];

    println!("Reading blocks.");

    let mut total_blocks = 0;

    loop {
        let mut buf = [0; 512];

        let bytes = input_file.read(&mut buf)?;

        if bytes < 512 {
            break;
        }

        let block = Block::from_bytes(&buf).map_err(Error::msg)?;

        binary.extend(&buf[0..(block.payload_size as usize)]);

        total_blocks += 1;
    }

    output_file.write(&binary)?;

    println!("Read {} bytes from {} blocks.", binary.len(), total_blocks);

    output_file.flush()?;

    Ok(())
}
