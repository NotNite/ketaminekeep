from litemapy import Schematic, Region, BlockState

schem = Schematic.load(
    "G:/Standalone/PrismLauncher/instances/1.20.2/.minecraft/schematics/jaybirthday3.litematic"
)
reg = schem.regions["Unnamed"]

blocks = []

for x in range(reg.minx(), reg.maxx() + 1):
    for y in range(reg.miny(), reg.maxy() + 1):
        for z in range(reg.minz(), reg.maxz() + 1):
            pos = (x, y, z)
            block = reg.getblock(*pos)
            if (
                block.blockid == "minecraft:air"
                or block.blockid == "minecraft:cave_air"
                or block.blockid == "minecraft:void_air"
            ):
                continue
            id = block.blockid.split(":")[1]
            props = block.to_block_state_identifier().replace(block.blockid, "")
            blocks.append({"pos": pos, "id": id, "props": props})

import json
import os

with open("../jaybirthday.json", "w") as f:
    json.dump(blocks, f)
