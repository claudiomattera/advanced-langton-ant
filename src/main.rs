// Copyright Claudio Mattera 2020.
// Distributed under the MIT License.
// See accompanying file License.txt, or online at
// https://opensource.org/licenses/MIT

use log::*;

use std::path::{Path, PathBuf};

use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version};
use clap::{Arg, ArgMatches, SubCommand};

mod board;
mod palette;

use crate::board::{Board, Rule};
use crate::palette::Palette;

fn main() {
    let matches = parse_arguments();
    setup_logging(matches.occurrences_of("verbosity"));

    if matches.subcommand().1.is_none() {
        println!("{}", matches.usage());
        return;
    }

    let n = matches
        .value_of("size")
        .expect("Missing argument \"size\"")
        .parse()
        .expect("Size must be an integer");
    let lifetime = matches
        .value_of("lifetime")
        .expect("Missing argument \"lifetime\"")
        .parse()
        .expect("Lifetime must be an integer");
    let rule = matches
        .value_of("rule")
        .map(Rule::new)
        .expect("Missing argument \"rule\"")
        .expect("Invalid rule string");

    info!("Creating grid {size}×{size}", size = n);
    let board = Board::new(n);

    info!("Creating palette with {} colors", rule.len());
    let palette = matches
        .value_of("palette")
        .map(|name| Palette::new(name, rule.len()))
        .expect("Missing argument \"palette\"")
        .expect("Invalid palette name");

    match matches.subcommand() {
        ("save-image", Some(subcommand)) => save_image(subcommand, lifetime, rule, board, palette),
        ("save-frames", Some(subcommand)) => save_frames(subcommand, lifetime, rule, board, palette),
        _ => unreachable!(),
    }
}

fn parse_arguments() -> ArgMatches<'static> {
    app_from_crate!()
        .after_help(
            "Some interesting rules:
 * LR (original highway)
 * LRRL (symmetric growth)
 * LRRRRRLLR (diagonal tubes)
 * RRLLLRLLLRRR (growing triangle)
 * RRLLLRRL (ninja star)
 * LRRRRLLLRRR (Archimedes' spiral)
 * LRRRRRRRLRLLRRRRRLRRRRRRLRLRLRLRLRLRLLRRLLRRLLR (more diagonal tubes)",
        )
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .arg(
            Arg::with_name("size")
                .short("s")
                .long("size")
                .default_value("100")
                .help("Grid size")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("lifetime")
                .short("l")
                .long("lifetime")
                .default_value("100000")
                .help("Ant lifetime")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("rule")
                .short("r")
                .long("rule")
                .default_value("LR")
                .help("Ant rule")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("palette")
                .short("p")
                .long("palette")
                .default_value("blue")
                .possible_values(&["blue", "red"])
                .help("Color palette")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("save-image")
                .about("Save final board status as an image")
                .arg(
                    Arg::with_name("image-path")
                        .short("p")
                        .long("path")
                        .required(true)
                        .help("Path to image")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("save-frames")
                .about("Save board progression as frames")
                .arg(
                    Arg::with_name("frames-directory-path")
                        .short("p")
                        .long("path")
                        .required(true)
                        .help("Path to frames directory")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("frames-count")
                        .short("c")
                        .long("count")
                        .default_value("900")
                        .help("Estimate number of frames")
                        .takes_value(true),
                ),
        )
        .get_matches()
}

fn setup_logging(verbosity: u64) {
    let default_log_filter = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "info,house_dashboard=debug",
        3 | _ => "debug",
    };
    let filter = env_logger::Env::default().default_filter_or(default_log_filter);
    env_logger::Builder::from_env(filter)
        .format_timestamp(None)
        .init();
}

fn save_image(
            subcommand: &ArgMatches,
            lifetime: u64,
            rule: Rule,
            mut board: Board,
            palette: Palette,
        ) {
    info!("Walking ant for {} steps using rule {}", lifetime, rule);
    board.walk_langton_ant(&rule, lifetime);

    let image = board.to_image(&palette);

    info!("Saving final state of the board");
    let image_path = subcommand
        .value_of("image-path")
        .map(Path::new)
        .expect("Missing argument \"image-path\"");
    match image.save(image_path) {
        Ok(_) => {}
        Err(error) => error!("Error: {}", error),
    }
}

fn save_frames(
            subcommand: &ArgMatches,
            lifetime: u64,
            rule: Rule,
            mut board: Board,
            palette: Palette,
        ) {
    let frames_directory_path = subcommand
        .value_of("frames-directory-path")
        .map(PathBuf::from)
        .expect("Missing argument frames-directory-path.");
    let frames_count = subcommand
        .value_of("frames-count")
        .expect("Missing argument \"frames-count\"")
        .parse::<u64>()
        .expect("Frames count must be an integer");

    let mut frame_index = 0;
    let frame_step = lifetime / frames_count;

    info!(
        "Walking ant for {} steps using rule {}, saving one frame every {} iterations",
        lifetime, rule, frame_step
    );
    board.walk_and_save_langton_ant(
        &rule,
        lifetime,
        Box::new(
            move |board, iteration| {
                if iteration % frame_step == 0 {
                    debug!("Saving {}th state of the board (of {})", frame_index, frames_count);
                    let frame = board.to_image(&palette);
                    let frame_path = frames_directory_path.join(format!("{:09}.png", frame_index));
                    match frame.save(frame_path) {
                        Ok(_) => {}
                        Err(error) => error!("Error: {}", error),
                    }
                    frame_index += 1;
                }
            })
    );
}
