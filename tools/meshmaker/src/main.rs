/*! Generate meshes for use as assets */

/*
 Author: Dorota Czaplejewicz <gihuac.dcz@porcupinefactory.org>
 SPDX-License-Identifier: AGPL-3.0-or-later
 */

use immense as im;
use immense::{ cube, write_meshes, ExportConfig, OutputMesh, Replicate, Rule, Tf, TransformArgument };
use immense::{ rule, tf };
use rand;
use std::cell::RefCell;
use std::fs::File;
use std::io;


use rand::seq::SliceRandom;


#[derive(Debug)]
enum Error {
    Immense(im::Error),
    Io(io::Error),
}

fn double_helix<R: rand::Rng>(rng: &mut R) -> impl Iterator<Item=OutputMesh> {
    // Commented out cause im::ToRule insists on being 'static.
    //struct BasePair<'a, R: rand::Rng>(RefCell<&'a mut R>);
    struct BasePair;
    
    //impl<R: rand::Rng> im::ToRule for BasePair<'static, R> {
    impl im::ToRule for BasePair {
        fn to_rule(&self) -> Rule {
            // let mut rng = *self.0.borrow_mut();
            let mut rng = rand::thread_rng();
            
            let at = rule![
                    tf![Tf::hue(50.0)] => rule![
                        tf![Tf::tx(1.5)] => rule![
                            tf![Tf::rx(80.0), Tf::ry(40.0), Tf::rz(0.0)] => cube(),
                        ],
                        tf![Tf::tx(0.5)] => rule![
                            tf![Tf::rx(70.0), Tf::ry(30.0), Tf::rz(10.0)] => cube(),
                        ],
                    ],
                    tf![Tf::hue(90.0)] => rule![
                        tf![Tf::tx(-1.5)] => rule![
                            tf![Tf::rx(70.0), Tf::ry(30.0), Tf::rz(-10.0)] => cube(),
                        ],
                        tf![Tf::tx(-0.5)] => rule![
                            tf![Tf::rx(60.0), Tf::ry(20.0), Tf::rz(0.0)] => cube(),
                        ],
                    ],
            ];
            let gc = rule![
                    tf![Tf::hue(150.0)] => rule![
                        tf![Tf::tx(1.5)] => rule![
                            tf![Tf::rx(40.0), Tf::ry(10.0), Tf::rz(80.0)] => cube(),
                        ],
                        tf![Tf::tx(0.5)] => rule![
                            tf![Tf::rx(30.0), Tf::ry(0.0), Tf::rz(90.0)] => cube(),
                        ],
                    ],
                    tf![Tf::hue(250.0)] => rule![
                        tf![Tf::tx(-0.5)] => rule![
                            tf![Tf::rx(50.0), Tf::ry(0.0), Tf::rz(0.0)] => cube(),
                        ],
                        tf![Tf::tx(-1.5)] => rule![
                            tf![Tf::rx(60.0), Tf::ry(10.0), Tf::rz(10.0)] => cube(),
                        ],
                    ],
            ];

            [
                at.clone(),
                gc.clone(),
                // reverse
                rule![
                    tf![Tf::ry(180.0)] => at,
                ],
                rule![
                    tf![Tf::ry(180.0)] => gc,
                ],
            ].choose(&mut rng)
            .unwrap()
            .to_owned()
        }
    }
    
    let stem = rule![
        tf![Tf::ry(18.0), Tf::ty(1.0)] => rule![
            tf![Tf::tx(3.0)] => rule![
                tf![Tf::rx(20.0), Tf::ry(50.0), Tf::rz(30.0)] => cube(),
            ],
        ],
        tf![Tf::tx(3.0)] => rule![
            tf![Tf::rx(10.0), Tf::ry(40.0), Tf::rz(20.0)] => cube(),
        ],
    ];
    
    let rule = rule![
        tf![Tf::saturation(0.3), Tf::hue(160.0)] => stem.clone(),
        tf![Tf::saturation(0.3), Tf::ry(-180.0)] => stem,
        tf![Tf::tx(0.0)] => BasePair, //BasePair(RefCell::new(rng)));
    ];
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
    let mut rng = rand::thread_rng();
    let rng = &mut rng;
    write_to_file(double_helix(rng), "helix.obj")
}
