# Ketamine Keep

This is a set of scripts I used to build [Ketamine Keep](https://notnite.com/blog/minecraft-to-goldsrc), a Half-Life 1 deathmatch map of a Minecraft schematic. It was a birthday present to [my girlfriend](https://hl2.sh/) (which she ended up insisting on helping with anyways).

**This code is not ready for use and never will be.** All of the assets are missing, several paths are hardcoded, there are several hardcoded cases for the specific schematic it was made for, there are a lot of bugs, and there was still plenty of manual work done. I'm putting this out here so people can have fun with it. Good luck if it even compiles!

## Project directory

- exporter.py: Takes in a .litematic file and converts it to .json.
- converter: A Rust program that takes the .json file and outputs a .vmf from scratch.
- ImageConverter: Converts textures to 8bpp BMP files.
- modelgen: The model conversion code from converter but as a standalone application. Designed for bulk model processing to ease rendering issues.

## Credits

- NotNite: Project "lead", converter code
- funcjay: Map cleanup, model recreation, deathmatchifying the map
- Project Orange: Map design (miss you all)
- Mojang: Texture and model assets

along with all of these tools and resources:

- [Prism Launcher](https://prismlauncher.org/) for getting the Minecraft 1.20.2 client jar
- [Litematica](https://github.com/maruohon/litematica) for generating the schematic from the world
- [Panoramica](https://modrinth.com/mod/panoramica) for taking the skybox photo
- [litemapy](https://pypi.org/project/litemapy/) for parsing JSON out of the schematic
- [paint.net](https://getpaint.net/) for texture editing
- [J.A.C.K.](https://jack.hlfx.ru/en/) for all of the manual work required after converting it
- [WadMaker](https://github.com/pwitvoet/wadmaker) for assembling the textures into a .wad
- [Blockbench](https://www.blockbench.net/) for converting Minecraft models to .obj
- [EnhancedBlockEntities](https://github.com/FoundationGames/EnhancedBlockEntities) for the chest model
- Sven Co-op's StudioMDL for compiling Minecraft models
- [VHLT](https://developer.valvesoftware.com/wiki/VHLT) for compiling the map
- Tons of information on [minecraft.wiki](https://minecraft.wiki/), [VDC](https://developer.valvesoftware.com/wiki/Main_Page), [TWHL](https://twhl.info/), and [the303.org](https://the303.org/)
