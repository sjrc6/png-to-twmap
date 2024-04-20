use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use image::{DynamicImage, GenericImageView, Pixel, Pixels};
use ndarray::Array2;
use serde::{Deserialize, Serialize};
use twmap::{GameLayer, GameTile, TileFlags, TilemapLayer, TwMap};

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    /// Prints the default mappings to stdout
    ExportMappings,
    /// Converts a png to a tw map
    Convert {
        #[arg(short, long)]
        /// The mappings to use for the conversion
        mappings: Option<PathBuf>,
        /// The path to the input png file
        input_png: PathBuf,
        /// The path where the output map will be saved
        output_map: PathBuf,
    },
}

pub fn export(
    path: &PathBuf,
    width: usize,
    height: usize,
    pixels: Pixels<DynamicImage>,
    mapping: &HashMap<[u8; 4], BlockType>,
) {
    let mut map = TwMap::parse(include_bytes!("../EMPTY.map")).expect("parsing failed");
    map.load().expect("loading failed");

    // get game layer
    let game_layer = map
        .find_physics_layer_mut::<GameLayer>()
        .unwrap()
        .tiles_mut()
        .unwrap_mut();

    *game_layer =
        Array2::<GameTile>::from_elem((width, height), GameTile::new(0, TileFlags::empty()));

    // modify game layer
    for (x, y, value) in pixels {
        let rgba = value.to_rgba();
        let channels = rgba.channels();
        game_layer[[y as usize, x as usize]] = GameTile::new(
            mapping
                .get(&[channels[0], channels[1], channels[2], channels[3]])
                .unwrap_or(&BlockType::Empty)
                .get_id(),
            TileFlags::empty(),
        );
    }

    // save map
    println!("exporting map to {:?}", &path);
    map.save_file(path).expect("saving failed");
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum BlockType {
    Empty,
    Hookable,
    Freeze,
    Spawn,
    Start,
    Finish,
    Custom(u8),
}

impl BlockType {
    fn get_id(self) -> u8 {
        match self {
            BlockType::Empty => 0,
            BlockType::Hookable => 1,
            BlockType::Freeze => 9,
            BlockType::Spawn => 192,
            BlockType::Start => 33,
            BlockType::Finish => 34,
            BlockType::Custom(id) => id,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Mapping {
    mapping: HashMap<[u8; 4], u8>,
}

const DEFAULT_MAPPING: &str = include_str!("../config.rsn");

fn main() {
    let args = Args::parse();
    match args.command {
        Command::ExportMappings => {
            println!("{DEFAULT_MAPPING}");
        }
        Command::Convert {
            mappings,
            input_png,
            output_map,
        } => {
            let mappings: HashMap<[u8; 4], BlockType> = mappings
                .map(|path| {
                    rsn::from_str(
                        &std::fs::read_to_string(path).expect("Config file doesn't exist"),
                    )
                    .expect("Failed to parse config")
                })
                .unwrap_or_else(|| rsn::from_str(DEFAULT_MAPPING).expect("Failed to parse config"));

            // Open the image file
            let img = image::open(input_png).expect("Failed to open image");

            // Get the dimensions of the image
            let (width, height) = img.dimensions();

            export(
                &output_map,
                width as usize,
                height as usize,
                img.pixels(),
                &mappings,
            );
        }
    }
}
