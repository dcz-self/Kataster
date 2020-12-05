/*! Generate meshes for use as assets */

/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */

use immense as im;
use immense::{ cube, write_meshes, ExportConfig, OutputMesh };
use std::fs::File;
use std::io;

#[derive(Debug)]
enum Error {
    Immense(im::Error),
    Io(io::Error),
}

fn double_helix() -> impl Iterator<Item=OutputMesh> {
    let rule = cube();
    rule.generate()
}

fn write_to_file(meshes: impl Iterator<Item=OutputMesh>, name: &str)
    -> Result<(), Error>
{
    let mut output_file = File::create(name).map_err(Error::Io)?;
    write_meshes(ExportConfig::default(), meshes, &mut output_file)
        .map_err(Error::Immense)?;
    Ok(())
}

fn main() -> Result<(), Error> {
    write_to_file(double_helix(), "helix.obj")
}
