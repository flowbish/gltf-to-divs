# gltf to divs converter

As seen on [https://borzoi.horse/horse](https://borzoi.horse/horse), I made a set of css styles can transform divs applied with certain custom properties into 3d space, ultimately rendering a 3d model. This tool exists to take a gltf model and export a hierarcy of divs corresponding to nodes and meshes in the model. 

Currently, this assumes that adjacent triangle pairs form a quad, which is an assumption that fell out of using this primarily to convert [Blockbench](https://www.blockbench.net/) models. In order to use this tool at the moment, your model must hold to the same assumption.

To use this tool:
1. Create a model with Blockbench, or import an existing Minecraft mob.
2. Export the model as a gltf file.
3. Run the tool

```bash
$ cargo run /path/to/model.gltf generate-divs --scale 100 > model.divs
```

The `model.divs` files can the be imported into your website, wrapped in the appropriate camera divs, and, with the web rendering stylesheet, you should have a 3d model.