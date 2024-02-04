use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct Config {
    min: (f64, f64, f64),
    max: (f64, f64, f64),
    origin: (f64, f64, f64),
    blocks: Vec<String>,
    single: bool,
    output_name: String,
}

#[derive(Debug, Deserialize)]
struct Block {
    pos: (i32, i32, i32),
    id: String,
    props: Option<String>,
}

const MC_TO_HAMMER: f64 = 48.;

fn fix_coords(pos: (f64, f64, f64)) -> (f64, f64, f64) {
    // Offset to fit in the map and convert to Hammer units (Z up)
    let pos = (pos.0, pos.2, pos.1);
    let offset = (0., -(4096. - 256.), -512.);
    (pos.0 + offset.0, pos.1 + offset.1, pos.2 + offset.2)
}

fn main() -> anyhow::Result<()> {
    let config: Config = serde_json::from_str(&std::fs::read_to_string("config.json")?)?;
    let schema: Vec<Block> = serde_json::from_str(include_str!("jaybirthday.json"))?;

    let min = (
        schema.iter().map(|b| b.pos.0).min().unwrap(),
        schema.iter().map(|b| b.pos.1).min().unwrap(),
        schema.iter().map(|b| b.pos.2).min().unwrap(),
    );
    let max = (
        schema.iter().map(|b| b.pos.0).max().unwrap(),
        schema.iter().map(|b| b.pos.1).max().unwrap(),
        schema.iter().map(|b| b.pos.2).max().unwrap(),
    );

    let mut all_positions = HashMap::new();

    if config.single {
        for block in config.blocks {
            all_positions.insert(block.clone(), vec![(0., 0., 0.)]);
        }
    } else {
        for block in schema {
            if !config.blocks.contains(&block.id) {
                continue;
            }

            // Offset the negative so it starts at zero
            let pos = (
                (block.pos.0 - min.0) as usize,
                (block.pos.1 - min.1) as usize,
                (block.pos.2 - min.2) as usize,
            );
            // Flip it on the Z axis because Hammer moment (janky hack to do it here)
            let pos = (pos.0, pos.1, (max.2 - min.2) as usize - pos.2);

            // Then scale up
            let pos = (
                pos.0 as f64 * MC_TO_HAMMER,
                pos.1 as f64 * MC_TO_HAMMER,
                pos.2 as f64 * MC_TO_HAMMER,
            );
            let pos = fix_coords(pos);

            if pos.0 < config.min.0
                || pos.1 < config.min.1
                || pos.2 < config.min.2
                || pos.0 > config.max.0
                || pos.1 > config.max.1
                || pos.2 > config.max.2
            {
                continue;
            }

            all_positions
                .entry(block.id)
                .or_insert_with(Vec::new)
                .push(pos);
        }
    }

    std::fs::create_dir_all("./models_out")?;
    let mut smd = r#"version 1
nodes
0 "root" -1
end
skeleton
time 0
0 0 0 0 0 0 0
end
triangles
"#
    .to_string();
    let mut qc = format!(
        r#"$modelname "{}.mdl"
$cd "."
$origin 0 0 0 -90
$body "studio" "{}"
"#,
        config.output_name, config.output_name
    );

    for (id, positions) in all_positions {
        let (models, materials) = tobj::load_obj(
            format!("./models/{}.obj", id),
            &tobj::LoadOptions {
                triangulate: true,
                ..Default::default()
            },
        )?;
        let materials = materials?;

        let mut material_files = Vec::new();

        for model in models {
            let mesh = model.mesh;
            let material = &materials[mesh.material_id.unwrap()];
            // Extract the diffuse texture to a bmp
            let diffuse_name = material.diffuse_texture.clone().unwrap();
            let bmp_name = diffuse_name.replace(".png", ".bmp");
            if !material_files.iter().any(|m| *m == bmp_name) {
                material_files.push(bmp_name.clone());
            }

            let image_converter = "./ImageConverter.exe";
            std::process::Command::new(image_converter)
                .arg(format!("./models/{}", diffuse_name))
                .stdout(std::process::Stdio::piped())
                .spawn()?
                .wait_with_output()?;

            for position in &positions {
                // Center
                let mut position = (
                    position.0 + (MC_TO_HAMMER / 2.),
                    position.1 + (MC_TO_HAMMER / 2.),
                    position.2 + (MC_TO_HAMMER / 2.),
                );

                // Offset to the origin
                position.0 -= config.origin.0;
                position.1 -= config.origin.1;
                position.2 -= config.origin.2;

                for idx in 0..mesh.indices.len() / 3 {
                    let mut face = format!("{}\n", diffuse_name.replace(".png", ".bmp"));
                    for vtx in 0..3 {
                        let mut pos = (
                            mesh.positions[(mesh.indices[(idx * 3) + vtx] * 3) as usize],
                            mesh.positions[(mesh.indices[(idx * 3) + vtx] * 3 + 1) as usize],
                            mesh.positions[(mesh.indices[(idx * 3) + vtx] * 3 + 2) as usize],
                        );
                        // HACK
                        pos.0 *= MC_TO_HAMMER as f32;
                        pos.1 *= MC_TO_HAMMER as f32;
                        pos.2 *= MC_TO_HAMMER as f32;

                        // Swap to Z-up
                        pos = (pos.0, pos.2, pos.1);

                        // Offset to this position
                        pos.0 += position.0 as f32;
                        pos.1 += position.1 as f32;
                        pos.2 += position.2 as f32;

                        let normal = (
                            mesh.normals[(mesh.normal_indices[(idx * 3) + vtx] * 3) as usize],
                            mesh.normals[(mesh.normal_indices[(idx * 3) + vtx] * 3 + 1) as usize],
                            mesh.normals[(mesh.normal_indices[(idx * 3) + vtx] * 3 + 2) as usize],
                        );
                        let uv = (
                            mesh.texcoords[(mesh.texcoord_indices[(idx * 3) + vtx] * 2) as usize],
                            mesh.texcoords
                                [(mesh.texcoord_indices[(idx * 3) + vtx] * 2 + 1) as usize],
                        );

                        face += &format!(
                            "0 {} {} {} {} {} {} {} {}\n",
                            pos.0, pos.1, pos.2, normal.0, normal.1, normal.2, uv.0, uv.1
                        );
                    }
                    smd += &face;
                }
            }
        }

        for material in material_files {
            qc += &format!("$texrendermode \"{}\" masked\n", material);
        }
    }

    smd += "end\n";
    qc += &format!(
        r#"$sequence "idle" {{
    "{}"
    fps 1
}}
"#,
        config.output_name
    );
    std::fs::write(format!("./models_out/{}.smd", config.output_name), smd)?;
    std::fs::write(format!("./models_out/{}.qc", config.output_name), qc)?;

    std::process::Command::new("./studiomdl.exe")
        .arg(format!("./{}.qc", config.output_name))
        .current_dir("./models_out")
        .stdout(std::process::Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    Ok(())
}
