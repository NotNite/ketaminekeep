use image::{codecs::bmp::BmpEncoder, ImageEncoder};
use serde::Deserialize;
use std::{collections::HashMap, fs::File, io::BufReader};
use util::Face;

mod greedy;
mod util;

const MC_TO_HAMMER: f64 = 48.;

#[derive(Debug, Deserialize)]
struct Block {
    pos: (i32, i32, i32),
    id: String,
    props: Option<String>,
}

fn fix_coords(pos: (f64, f64, f64)) -> (f64, f64, f64) {
    // Offset to fit in the map and convert to Hammer units (Z up)
    let pos = (pos.0, pos.2, pos.1);
    let offset = (0., -(4096. - 256.), -512.);
    (pos.0 + offset.0, pos.1 + offset.1, pos.2 + offset.2)
}

fn build_brush(
    id: &mut usize,
    r#box: &greedy::Box,
    voxels: &HashMap<(usize, usize, usize), greedy::Voxel>,
    textures: &[String],
    missing_textures: &mut Vec<String>,
) -> String {
    let mut brush = format!(
        r#"
  solid
  {{
    "id" "{}"
"#,
        id
    );
    *id += 1;

    // They're always cubes, build six sides ez
    let min = (
        r#box.min.0 as f64 * MC_TO_HAMMER,
        r#box.min.1 as f64 * MC_TO_HAMMER,
        r#box.min.2 as f64 * MC_TO_HAMMER,
    );
    let max = (
        // Because we're scaling cubes to brushes, add 1 to the max
        (r#box.max.0 + 1) as f64 * MC_TO_HAMMER,
        (r#box.max.1 + 1) as f64 * MC_TO_HAMMER,
        (r#box.max.2 + 1) as f64 * MC_TO_HAMMER,
    );

    let min = fix_coords(min);
    let mut max = fix_coords(max);
    let mut missing_any = false;

    // Dirty hack: shrink path blocks
    if r#box.id == "dirt_path" {
        max.2 -= MC_TO_HAMMER / 16.;
    }

    let props = util::parse_properties(&r#box.properties);
    let face = match props.get("facing").map(|s| s.as_str()).unwrap_or("north") {
        "north" => Face::North,
        "east" => Face::East,
        "south" => Face::South,
        "west" => Face::West,
        "up" => Face::Top,
        "down" => Face::Bottom,
        _ => unreachable!(),
    };

    for side_id in 0..6 {
        // I'm gonna let copilot write this one. Here's some VDC quotes:

        // The plane is defined using these three points. The first marks the bottom left of the face, the second marks the top left, and the third marks the top right.
        // "plane" "(256 -256 160) (256 -256 -0) (256 256 160)"
        // "plane" "(288 256 160) (288 256 -0) (288 -256 160)"
        // "plane" "(256 256 160) (256 256 -0) (288 256 160)"
        // "plane" "(288 -256 160) (288 -256 -0) (256 -256 160)"
        // "plane" "(256 256 0) (256 -256 0) (288 256 0)"
        // "plane" "(256 -256 160) (256 256 160) (288 -256 160)"

        // TODO: adjust texture based on face
        let texture = match r#box.id.as_str() {
            "tinted_glass" => "SKY".to_string(),
            "barrier" => "CLIP".to_string(),
            "light_blue_concrete" => "lbconc".to_string(),
            "chiseled_polished_blackstone" => "cpbs".to_string(),
            "cracked_polished_blackstone_bricks" => "cpbsb".to_string(),
            "gilded_blackstone" => "gbs".to_string(),
            "polished_blackstone" => "pbs".to_string(),
            "polished_blackstone_bricks" => "pbsb".to_string(),
            "stripped_spruce_log" => "ssl".to_string(),
            "white_terracotta" => "wt".to_string(),
            "light_gray_terracotta" => "lgt".to_string(),
            "light_blue_terracotta" => "lbt".to_string(),
            "chiseled_stone_bricks" => "csb".to_string(),

            "glass" => "{glass".to_string(),
            "glass_pane" => "{glass".to_string(),
            "blue_stained_glass" => "{bsg".to_string(),
            "oak_leaves" => "{oak_leaves".to_string(),
            "spruce_leaves" => "{spruce_leaves".to_string(),

            "grass_block" => {
                if side_id == Face::Top as usize {
                    "grass_block".to_string()
                } else if side_id == Face::Bottom as usize {
                    "dirt".to_string()
                } else {
                    "gbs2".to_string()
                }
            }

            "crafting_table" => {
                if side_id == 5 {
                    "ct_top".to_string()
                } else if side_id == face as usize {
                    "ct_front".to_string()
                } else {
                    "ct_side".to_string()
                }
            }

            "furnace" => {
                if side_id == 5 {
                    "furnace_top".to_string()
                } else if side_id == face as usize {
                    "furnace_front".to_string()
                } else {
                    "furnace_side".to_string()
                }
            }
            "blast_furnace" => {
                if side_id == 5 {
                    "bf_top".to_string()
                } else if side_id == face as usize {
                    "bf_front".to_string()
                } else {
                    "bf_side".to_string()
                }
            }
            "smoker" => {
                if side_id == Face::Top as usize {
                    "smoker_top".to_string()
                } else if side_id == Face::Bottom as usize {
                    "smoker_bottom".to_string()
                } else if side_id == face as usize {
                    "smoker_front".to_string()
                } else {
                    "smoker_side".to_string()
                }
            }

            "barrel" => {
                // TODO
                if side_id == Face::Top as usize {
                    "barrel_top".to_string()
                } else if side_id == Face::Bottom as usize {
                    "barrel_bottom".to_string()
                } else {
                    "barrel_side".to_string()
                }
            }

            "oak_log" => {
                if side_id == Face::Top as usize || side_id == Face::Bottom as usize {
                    "oak_log_top".to_string()
                } else {
                    "oak_log".to_string()
                }
            }
            "birch_log" => {
                if side_id == Face::Top as usize || side_id == Face::Bottom as usize {
                    "birch_log_top".to_string()
                } else {
                    "birch_log".to_string()
                }
            }
            "stripped_dark_oak_log" => {
                if side_id == Face::Top as usize || side_id == Face::Bottom as usize {
                    "sdolt".to_string()
                } else {
                    "sdol".to_string()
                }
            }

            _ => {
                if textures.iter().any(|t| *t == format!("{}.png", r#box.id)) {
                    r#box.id.clone()
                } else {
                    if !missing_textures.iter().any(|t| *t == r#box.id) {
                        missing_textures.push(r#box.id.clone());
                    }
                    missing_any = true;
                    "MISSING".to_string()
                }
            }
        };

        let plane_one = match side_id {
            0 => (min.0, min.1, max.2),
            1 => (max.0, max.1, max.2),
            2 => (min.0, max.1, max.2),
            3 => (max.0, min.1, max.2),
            4 => (min.0, max.1, min.2),
            5 => (min.0, min.1, max.2),
            _ => unreachable!(),
        };
        let plane_two = match side_id {
            0 => (min.0, min.1, min.2),
            1 => (max.0, max.1, min.2),
            2 => (min.0, max.1, min.2),
            3 => (max.0, min.1, min.2),
            4 => (min.0, min.1, min.2),
            5 => (min.0, max.1, max.2),
            _ => unreachable!(),
        };
        let plane_three = match side_id {
            0 => (min.0, max.1, max.2),
            1 => (max.0, min.1, max.2),
            2 => (max.0, max.1, max.2),
            3 => (min.0, min.1, max.2),
            4 => (max.0, max.1, min.2),
            5 => (max.0, min.1, max.2),
            _ => unreachable!(),
        };

        let uv_scale = 48. / 16.;
        // Side faces only
        let v_offset = if side_id == 4 || side_id == 5 {
            0.
        } else {
            5.35
        };

        let side = format!(
            r#"
    side
    {{
      "id" "{}"
      "plane" "({} {} {}) ({} {} {}) ({} {} {})"
      "material" "{}"
      "uaxis" "[1 0 0 0] {}"
      "vaxis" "[0 -1 0 {}] {}"
      "rotation" "0"
      "lightmapscale" "0"
      "smoothing_groups" "0"
    }}
"#,
            id,
            plane_one.0,
            plane_one.1,
            plane_one.2,
            //
            plane_two.0,
            plane_two.1,
            plane_two.2,
            //
            plane_three.0,
            plane_three.1,
            plane_three.2,
            //
            texture,
            uv_scale,
            v_offset,
            uv_scale
        );
        *id += 1;
        brush = brush + &side;
    }

    brush += format!(
        r#"
    editor
    {{
      "color" "0 255 0"
      "visgroupid" "{}"
      "visgroupshown" "1"
    }}
  }}"#,
        if missing_any { "4" } else { "1" }
    )
    .as_str();

    brush
}

fn convert_obj_to_smd(model_name: String, exported_models: &mut [String]) -> anyhow::Result<()> {
    if exported_models.iter().any(|m| *m == model_name) {
        return Ok(());
    }

    let (models, materials) = tobj::load_obj(
        format!("../models/{}.obj", model_name),
        &tobj::LoadOptions {
            triangulate: true,
            ..Default::default()
        },
    )?;
    let materials = materials?;

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

        let image_converter = "W:/bullshit/projects/jaybirthday/ImageConverter/ImageConverter/bin/Debug/net8.0-windows/ImageConverter.exe";
        std::process::Command::new(image_converter)
            .arg(format!("../models/{}", diffuse_name))
            .current_dir("../models")
            .stdout(std::process::Stdio::piped())
            .spawn()?
            .wait_with_output()?;

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

                let normal = (
                    mesh.normals[(mesh.normal_indices[(idx * 3) + vtx] * 3) as usize],
                    mesh.normals[(mesh.normal_indices[(idx * 3) + vtx] * 3 + 1) as usize],
                    mesh.normals[(mesh.normal_indices[(idx * 3) + vtx] * 3 + 2) as usize],
                );
                let uv = (
                    mesh.texcoords[(mesh.texcoord_indices[(idx * 3) + vtx] * 2) as usize],
                    mesh.texcoords[(mesh.texcoord_indices[(idx * 3) + vtx] * 2 + 1) as usize],
                );

                face += &format!(
                    "0 {} {} {} {} {} {} {} {}\n",
                    pos.0, pos.1, pos.2, normal.0, normal.1, normal.2, uv.0, uv.1
                );
            }
            smd += &face;
        }
    }

    smd += "end\n";
    std::fs::write(format!("../models_out/{}.smd", model_name), smd)?;
    let mut qc = format!(
        r#"$modelname "{}.mdl"
$cd "."
$body "studio" "{}"
"#,
        model_name, model_name
    );

    for material in material_files {
        qc += &format!("$texrendermode \"{}\" masked\n", material);
    }
    qc += &format!(
        r#"$sequence "idle" {{
    "{}"
    fps 1
}}
"#,
        model_name
    );

    std::fs::write(format!("../models_out/{}.qc", model_name), qc)?;

    let studiomdl = "W:/bullshit/projects/jaybirthday/studiomdl.exe";
    std::process::Command::new(studiomdl)
        .arg(format!("./{}.qc", model_name))
        .current_dir("../models_out")
        .stdout(std::process::Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    let models = "E:/SteamLibrary/steamapps/common/Half-Life/birthday/models";
    std::fs::copy(
        format!("../models_out/{}.mdl", model_name),
        format!("{}/{}.mdl", models, model_name),
    )?;
    Ok(())
}

fn build_model(
    id: &mut usize,
    pos: (usize, usize, usize),
    voxel: &greedy::Voxel,
    exported_models: &mut Vec<String>,
) -> Option<String> {
    let pos = (
        (pos.0 as f64 * MC_TO_HAMMER) + (MC_TO_HAMMER / 2.),
        (pos.1 as f64 * MC_TO_HAMMER) + (MC_TO_HAMMER / 2.),
        (pos.2 as f64 * MC_TO_HAMMER) + (MC_TO_HAMMER / 2.),
    );
    let pos = fix_coords(pos);

    let props = util::parse_properties(&voxel.properties);
    let model = match voxel.id.as_str() {
        "campfire" => Some("campfire"),
        "grass" => Some("grass"),
        "fire" => {
            if props
                .values()
                .filter(|x| **x == "true" || **x == "false")
                .all(|x| x == "false")
            {
                Some("fire_floor")
            } else {
                None
            }
        }
        v if v.starts_with("potted_") => Some("flower_pot"),
        _ => Some(voxel.id.as_str()),
    }?;

    if !std::path::Path::new(&format!("../models/{}.obj", model)).exists() {
        return None;
    }

    if !exported_models.iter().any(|m| *m == model) {
        convert_obj_to_smd(model.to_string(), exported_models).unwrap();
        exported_models.push(model.to_string());
    }

    let entity = format!(
        r#"
entity
{{
  "id" "{}"
  "classname" "env_sprite"
  "origin" "{} {} {}"
  "model" "models/{}.mdl"
  "angles" "-90 0 0"
  editor
  {{
    "visgroupid" "5"
  }}
}}
"#,
        id, pos.0, pos.1, pos.2, model
    );
    *id += 1;
    Some(entity)
}

fn main() -> anyhow::Result<()> {
    let schema: Vec<Block> = serde_json::from_str(&std::fs::read_to_string("jaybirthday.json")?)?;
    let mut voxels: HashMap<(usize, usize, usize), greedy::Voxel> = HashMap::new();
    let mut models: HashMap<(usize, usize, usize), greedy::Voxel> = HashMap::new();
    let mut torches: Vec<(usize, usize, usize)> = Vec::new();

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

    for block in schema {
        if block.id.contains("door") {
            continue;
        }
        if block.id.contains("sign") {
            continue;
        }
        if block.id.contains("ladder") {
            continue;
        }
        if block.id.contains("water") || block.id.contains("lava") {
            continue;
        }

        // Offset the negative so it starts at zero
        let pos = (
            (block.pos.0 - min.0) as usize,
            (block.pos.1 - min.1) as usize,
            (block.pos.2 - min.2) as usize,
        );

        // JANK JANK JANK
        if pos == (21, 37, 113)
            || pos == (31, 30, 33)
            || pos == (21, 35, 113)
            || pos == (31, 27, 33)
            || pos == (4, 33, 36)
            || pos == (12, 22, 30)
        {
            continue;
        }
        let old_pos = pos;

        // Flip it on the Z axis because Hammer moment (janky hack to do it here)
        let pos = (pos.0, pos.1, (max.2 - min.2) as usize - pos.2);

        if block.id == "campfire"
            || block.id == "grass"
            || [
                "oxeye_daisy",
                "cornflower",
                "azure_bluet",
                "poppy",
                "dandelion",
                "oak_pressure_plate",
                "stone_pressure_plate",
                "heavy_weighted_pressure_plate",
                "light_weighted_pressure_plate",
                "flower_pot",
                "fire",
                "lectern",
            ]
            .contains(&block.id.as_str())
            || block.id.starts_with("potted_")
        {
            models.insert(
                pos,
                greedy::Voxel::new(block.id, block.props.unwrap_or("".to_string())),
            );
            continue;
        }

        if block.id.contains("torch") || block.id.contains("lava") {
            torches.push(pos);
            continue;
        }

        let mut props = block.props.clone().unwrap_or("".to_string());
        if block.id.contains("leaves") {
            props = "".to_string();
        }

        voxels.insert(pos, greedy::Voxel::new(block.id, props));
    }

    let spawn = (22., 34.3, 19.);
    let camera = (spawn.0, spawn.1 + 5., spawn.2);
    let light = (spawn.0, spawn.1 + 10., spawn.2);
    let spawn = fix_coords((
        spawn.0 * MC_TO_HAMMER,
        spawn.1 * MC_TO_HAMMER,
        spawn.2 * MC_TO_HAMMER,
    ));
    let camera = fix_coords((
        camera.0 * MC_TO_HAMMER,
        camera.1 * MC_TO_HAMMER,
        camera.2 * MC_TO_HAMMER,
    ));
    let light = fix_coords((
        light.0 * MC_TO_HAMMER,
        light.1 * MC_TO_HAMMER,
        light.2 * MC_TO_HAMMER,
    ));

    let mut id = 0;
    let mut vmf = format!(
        r#"
versioninfo
{{
  "editorversion" "400"
  "editorbuild" "2959"
  "mapversion" "1"
  "formatversion" "100"
  "prefab" "0"
}}
viewsettings
{{
  "bSnapToGrid" "1"
  "bShowGrid" "1"
  "bShow3DGrid" "0"
  "nGridSpacing" "2"
}}
cameras
{{
  "activecamera" "0"
  camera
  {{
    "position" "[{} {} {}]"
    "look" "[0 90 0]"
  }}
}}
cordon
{{
  "mins" "(0 0 0)"
  "maxs" "(0 0 0)"
  "active" "0"
}}
"#,
        camera.0, camera.1, camera.2
    );

    vmf += r#"
visgroups
{
  visgroup
  {
    "name" "Generated World"
    "visgroupid" "1"
    "color" "255 255 255"
    "visible" "1"
  }

  visgroup
  {
    "name" "Entities"
    "visgroupid" "2"
    "color" "255 255 255"
    "visible" "1"
  }

  visgroup
  {
    "name" "Lights"
    "visgroupid" "3"
    "color" "255 255 255"
    "visible" "1"
  }

  visgroup
  {
    "name" "Missing Textures"
    "visgroupid" "4"
    "color" "255 255 255"
    "visible" "1"
  }

  visgroup
  {
    "name" "Models"
    "visgroupid" "5"
    "color" "255 255 255"
    "visible" "1"
  }
}
"#;

    vmf = vmf
        + &format!(
            r#"
entity
{{
  "id" "{}"
  "classname" "info_player_start"
  "origin" "{} {} {}"
  "angles" "0 90 0"
  editor
  {{
    "color" "0 255 0"
    "visgroupid" "2"
    "visgroupshown" "1"
  }}
}}
"#,
            id,
            spawn.0 - (MC_TO_HAMMER / 2.),
            spawn.1 - (MC_TO_HAMMER / 2.),
            spawn.2 - (MC_TO_HAMMER / 2.)
        );
    id += 1;

    vmf = vmf
        + &format!(
            r#"
entity
{{
  "id" "{}"
  "classname" "light_environment"
  "origin" "{} {} {}"
  "_light" "240 240 255 170"
  "pitch" "-90"
  editor
  {{
    "color" "220 30 220"
    "visgroupid" "3"
    "visgroupshown" "1"
  }}
}}
"#,
            id, light.0, light.1, light.2
        );
    id += 1;
    vmf = vmf
        + &format!(
            r#"
entity
{{
  "id" "{}"
  "classname" "info_texlights"
  "origin" "{} {} {}"
  "glowstone" "171 131 83 1000"
}}
"#,
            id, light.0, light.1, light.2
        );
    id += 1;

    let mut world = format!(
        r#"
world
{{
  "id" "{}"
  "mapversion" "1"
  "classname" "worldspawn"
  "_generator" "absolute gangstas hacker technology"
  "defaultteam" "0"
  "newunit" "0"
  "gametitle" "0"
  "startdark" "0"
  "MaxRange" "8192"
  "sounds" "1"
  "skyname" "jaymc"
"#,
        id
    );
    id += 1;

    // Merge brushes together - a wall of the same block should be one continuous brush
    let boxes = greedy::best_greedy(&voxels);
    println!("{} boxes", boxes.len());

    let mut fills: Vec<String> = Vec::new();
    for r#box in &boxes {
        let fill_command = format!(
            "fill {} {} {} {} {} {} {}",
            r#box.min.0, r#box.min.1, r#box.min.2, r#box.max.0, r#box.max.1, r#box.max.2, r#box.id
        );
        fills.push(fill_command);
    }
    std::fs::write("fills.txt", fills.join("\n"))?;

    let textures = std::fs::read_dir("../textures")?
        .flat_map(|r| r.map(|e| e.file_name()))
        .map(|s| s.to_str().unwrap().to_string())
        .collect::<Vec<_>>();
    let mut missing_textures = Vec::new();

    let mut queued_entities = Vec::new();

    for r#box in boxes {
        if r#box.id.contains("glass") && !r#box.id.contains("tinted") {
            let mut entity = format!(
                r#"
    entity
    {{
        "id" "{}"
        "classname" "func_breakable"
        "rendermode" "2"
        "renderamt" "255"
        "health" "25"
        "spawnflags" "256"
        "zhlt_embedlightmap" "1"
"#,
                id
            );
            id += 1;

            entity += &build_brush(&mut id, &r#box, &voxels, &textures, &mut missing_textures);
            entity += "\n    }\n";
            queued_entities.push(entity);
        } else if r#box.id.contains("leaves") {
            let mut entity = format!(
                r#"
    entity
    {{
        "id" "{}"
        "classname" "func_illusionary"
        "rendermode" "4"
        "renderamt" "255"
        "zhlt_lightflags" "2"
"#,
                id
            );
            id += 1;

            entity += &build_brush(&mut id, &r#box, &voxels, &textures, &mut missing_textures);
            entity += "\n    }\n";
            queued_entities.push(entity);
        } else {
            world =
                world + &build_brush(&mut id, &r#box, &voxels, &textures, &mut missing_textures);
        }
    }

    world += "\n}\n";

    for torch in torches {
        let pos = (
            (torch.0 as f64 * MC_TO_HAMMER) + (MC_TO_HAMMER / 2.),
            (torch.1 as f64 * MC_TO_HAMMER) + (MC_TO_HAMMER / 2.),
            (torch.2 as f64 * MC_TO_HAMMER) + (MC_TO_HAMMER / 2.),
        );
        let pos = fix_coords(pos);
        let torch = format!(
            r#"
    entity
    {{
      "id" "{}"
      "classname" "light"
      "origin" "{} {} {}"
      "angles" "0 0 0"
      "_falloff" "0"
      "_fade" "1.0"
      "style" "0"
      "_light" "255 255 255 100"
      "light" "255 255 255 100"

      editor
      {{
        "color" "0 255 0"
        "visgroupid" "3"
        "visgroupshown" "1"
      }}
    }}
"#,
            id, pos.0, pos.1, pos.2
        );
        id += 1;
        queued_entities.push(torch);
    }

    let mut exported_models = Vec::new();
    for (pos, voxel) in models {
        if let Some(model) = build_model(&mut id, pos, &voxel, &mut exported_models) {
            queued_entities.push(model);
        }
    }

    for entity in queued_entities {
        vmf += &entity;
        vmf += "\n";
    }

    vmf = vmf + "\n" + &world;
    std::fs::write("jaybirthday.vmf", vmf)?;
    std::fs::write("missing.txt", missing_textures.join("\n"))?;

    Ok(())
}
