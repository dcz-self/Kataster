/*! Generate meshes for use as assets */

/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */

use immense as im;
use immense::{ cube, write_meshes, ExportConfig, OutputMesh, Replicate, Rule, Tf };
use std::fs::File;
use std::io;

#[derive(Debug)]
enum Error {
    Immense(im::Error),
    Io(io::Error),
}

fn double_helix() -> impl Iterator<Item=OutputMesh> {
    let rule = Rule::new()
        .push(vec![Tf::saturation(0.3), Tf::hue(160.0), Tf::tx(3.0)], cube())
        .push(vec![Tf::saturation(0.3), Tf::tx(-3.0)], cube());
    let rule = Rule::new()
        .push(
            Replicate::n(
                100,
                vec![Tf::ry(36.0), Tf::ty(2.0)],
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
    write_to_file(double_helix(), "helix.obj")
}
