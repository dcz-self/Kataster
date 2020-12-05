/*! Generate meshes for use as assets */

/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */

use immense as im;
use immense::{ cube, write_meshes, ExportConfig, OutputMesh, Replicate, Rule, Tf, TransformArgument };
use immense::{ rule, tf };
use rand;
use std::fs::File;
use std::io;


use rand::seq::SliceRandom;


#[derive(Debug)]
enum Error {
    Immense(im::Error),
    Io(io::Error),
}

fn double_helix<R: rand::Rng>(rng: &mut R) -> impl Iterator<Item=OutputMesh> {
    fn basepair<R: rand::Rng>(rng: &mut R) -> Rule {
        [
            rule![
                tf![Tf::tx(1.0), Tf::hue(50.0)] => cube(), // A
                tf![Tf::tx(-1.0), Tf::hue(90.0)] => cube(), // T
            ],
            rule![
                tf![Tf::tx(1.0), Tf::hue(150.0)] => cube(), // G
                tf![Tf::tx(-1.0), Tf::hue(250.0)] => cube(), // C
            ],
            // reverse
            rule![
                tf![Tf::tx(1.0), Tf::hue(90.0)] => cube(),
                tf![Tf::tx(-1.0), Tf::hue(50.0)] => cube(),
            ],
            rule![
                tf![Tf::tx(1.0), Tf::hue(250.0)] => cube(),
                tf![Tf::tx(-1.0), Tf::hue(150.0)] => cube(),
            ]
        ].choose(rng).unwrap().to_owned()
    }
    
    let rule = Rule::new()
        .push(vec![Tf::saturation(0.3), Tf::hue(160.0), Tf::tx(3.0)], cube())
        .push(vec![Tf::saturation(0.3), Tf::tx(-3.0)], cube())
        .push(Tf::tx(0.0), basepair(rng));
    let rule = Rule::new()
        .push(
            Replicate::n(
                100,
                vec![Tf::ry(36.0), Tf::ty(2.0)], // about 10 pairs to a full spin
            ),
            rule,
        );
    rule.generate()
}

fn write_to_file(meshes: impl Iterator<Item=OutputMesh>, name: &str)
    -> Result<(), Error>
{
    let mut output_file = File::create(name).map_err(Error::Io)?;
    write_meshes(
        ExportConfig {
            export_colors: Some(format!("{}.mtl", name)),
            ..ExportConfig::default()
        },
        meshes,
        &mut output_file,
    ).map_err(Error::Immense)?;
    Ok(())
}

fn main() -> Result<(), Error> {
    write_to_file(double_helix(&mut rand::thread_rng()), "helix.obj")
}
