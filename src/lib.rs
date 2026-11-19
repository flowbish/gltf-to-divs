use gltf::buffer;
use gltf::image;
use gltf::Material;
use gltf::image::Source;
use gltf::Document;

use wasm_bindgen::prelude::*;

#[derive(Debug, Clone)]
struct Node {
    name: String,
    transforms: Transforms,
}

pub struct Generator {
    nodes: Vec<Node>,
    output: String,
    scale: f32
}

impl Generator {
    pub fn new(scale: f32) -> Self {
        Self {
            nodes: Vec::new(),
            output: String::new(),
            scale,
        }
    }

    fn add_output(&mut self, output: &str) {
        for line in output.lines() {
            for _ in &self.nodes {
                self.output.push_str("  ");
            }
            self.output.push_str(line);
            self.output.push('\n');
        }
    }

    pub fn output(&self) -> &str {
        &self.output
    }

    fn push_node(&mut self, node: Node) {
        let name = &node.name;

        self.add_output(&format!(r#"<div class="node {name}" style=""#));
        self.add_output(&format!("  transform:"));

        let [x, y, z] = node.transforms.translation;
        let scale = self.scale;
        self.add_output(&format!("    translate3d("));
        self.add_output(&format!("      calc({x} * {scale} * 1px + var(--node-translate-x)),"));
        self.add_output(&format!("      calc({y} * {scale} * -1px + var(--node-translate-y)),"));
        self.add_output(&format!("      calc({z} * {scale} * 1px + var(--node-translate-z))"));
        self.add_output(&format!("    )"));

        let [x, y, z, w] = node.transforms.rotation;
        let y = -1.0 * y;
        self.add_output(&format!("    rotate3d({x}, {y}, {z}, calc(-2 * acos({w})))"));

        // Custom animatable rotation
        self.add_output(&format!("    rotate3d(var(--node-rotation-x), var(--node-rotation-y), var(--node-rotation-z), var(--node-rotation-angle))"));

        let [x, y, z] = node.transforms.scale;
        self.add_output(&format!("    scale3d(calc({x} * var(--node-scale)), calc({y} * var(--node-scale)), calc({z} * var(--node-scale)));"));

        self.add_output(&format!(r#"">"#));
        self.nodes.push(node);
    }

    fn pop_node(&mut self) {
        self.nodes.pop();
        self.add_output("</div>");
    }

    fn visit_face(&mut self, face: &Face, class: &str) {
        let scale = self.scale;
        let a_x = face.b.position[0] * scale;
        let a_y = face.b.position[1] * -scale;
        let a_z = face.b.position[2] * scale;
        let b_x = face.a.position[0] * scale;
        let b_y = face.a.position[1] * -scale;
        let b_z = face.a.position[2] * scale;
        let c_x = face.c.position[0] * scale;
        let c_y = face.c.position[1] * -scale;
        let c_z = face.c.position[2] * scale;
        let color = face.color.to_string();
        let texture_image = face.uri.clone().unwrap_or("texture.webp".to_string());
        let tex_a_s = face.b.texture[0];
        let tex_a_t = face.b.texture[1];
        let tex_b_s = face.a.texture[0];
        let tex_b_t = face.a.texture[1];
        let tex_c_s = face.c.texture[0];
        let tex_c_t = face.c.texture[1];
        self.add_output(&format!(r#"<div class="{class}" style="
    --a-x: {a_x};
    --a-y: {a_y};
    --a-z: {a_z};
    --b-x: {b_x};
    --b-y: {b_y};
    --b-z: {b_z};
    --c-x: {c_x};
    --c-y: {c_y};
    --c-z: {c_z};
    --texture-background-color: {color};
    --texture-image: url({texture_image});
    --texture-a-s: {tex_a_s};
    --texture-a-t: {tex_a_t};
    --texture-b-s: {tex_b_s};
    --texture-b-t: {tex_b_t};
    --texture-c-s: {tex_c_s};
    --texture-c-t: {tex_c_t};
"></div>"#));
    }

    fn visit_rectangle(&mut self,  rectangle: &Rectangle) {
        self.visit_face(&rectangle.face, "rect")
    }

    fn visit_triangle(&mut self, triangle: &Triangle) {
        self.visit_face(&triangle.face, "tri")

    }
}

#[derive(Debug, Clone)]
struct Transforms {
    translation: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Position{
    position: [f32; 3],
    texture: [f32; 2],
}


#[derive(Debug, Clone, Copy, PartialEq)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Face {
    a: Position,
    b: Position,
    c: Position,
    uri: Option<String>,
    color: Color,
}

#[derive(Debug, Clone, PartialEq)]
struct Triangle {
    face: Face,
}

impl Triangle {
    fn merge(&self, other: &Triangle) -> Option<Rectangle> {
        if self.face.b == other.face.a && self.face.c == other.face.c {
            Some(Rectangle { face: self.face.clone() })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
struct Rectangle {
    face: Face,
}

fn make_triangles(material: Material<'_>, positions: &Vec<[f32; 3]>, indices: &Vec<u32>, tex_coords: &Vec<[f32; 2]>) -> Vec<Triangle> {
    let [rf, gf, bf, af] = material.pbr_metallic_roughness().base_color_factor();
    let [r, g, b, a] = [(rf * 255f32) as u8, (gf * 255f32) as u8, (bf * 255f32) as u8, (af * 255f32) as u8];
    let color = Color { r, g, b, a };

    let uri = material.pbr_metallic_roughness().base_color_texture().and_then(
        |info| info.texture().source()
    ).and_then(|image| {
        match image.source() {
            Source::Uri { uri, .. } => {
                Some(uri.to_string())
            }
            _ => None,
        }
    });

    indices
        .as_slice()
        .chunks(3)
        .map(|chunk| {
            let a = chunk[0] as usize;
            let b = chunk[1] as usize;
            let c = chunk[2] as usize;
            Triangle {
                face: Face {
                    a: Position{position: positions[a], texture: tex_coords[a]},
                    b: Position{position: positions[b], texture: tex_coords[b]},
                    c: Position{position: positions[c], texture: tex_coords[c]},
                    uri: uri.clone(),
                    color,
                }
            }
        })
        .collect()
}

fn sort_rectangles(triangles: Vec<Triangle>) -> Vec<Rectangle> {
    let mut rectangles = vec![];

    let tri_pairs: Vec<(Triangle, Triangle)> = triangles
        .as_slice()
        .chunks(2)
        .map(|chunk| (chunk[0].clone(), chunk[1].clone()))
        .collect();

    for (left, right) in tri_pairs {
        if let Some(rect) = left.merge(&right) {
            rectangles.push(rect);
        } else {
            eprintln!("Failed to merge");
        }
    }
    rectangles
}

fn visit_children<'a>(generator: &mut Generator, node: &gltf::Node<'a>, buffers: &Vec<buffer::Data>, desired_nodes: &Vec<String>, quadify: bool) {
    let name = node.name().unwrap_or("N/A");

    if !desired_nodes.is_empty() && !desired_nodes.contains(&name.to_string()) {
        return;
    }
    
    let (translation, rotation, scale) = node.transform().decomposed();
    let transforms = Transforms { translation, rotation, scale };

    generator.push_node(Node{
        name: name.to_string(),
        transforms,
    });

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let positions = reader
                .read_positions()
                .map(|iter| iter.collect::<Vec<[f32; 3]>>())
                .unwrap_or_else(|| Vec::new());
            let indices = reader
                .read_indices()
                .map(|iter| iter.into_u32().collect::<Vec<u32>>())
                .unwrap_or_else(|| Vec::new());
            let tex_coords = reader
                .read_tex_coords(0)
                .map(|iter| iter.into_f32().collect::<Vec<[f32; 2]>>())
                .unwrap_or_else(|| Vec::new());
            let triangles = make_triangles(primitive.material(), &positions, &indices, &tex_coords);
            if quadify {
                let rectangles = sort_rectangles(triangles);
                for rectangle in &rectangles {
                    generator.visit_rectangle(rectangle);
                }
            } else {
                for triangle in &triangles {
                    generator.visit_triangle(triangle);
                }
            }
        }
    }

    for child in node.children() {
        visit_children(generator, &child, buffers, desired_nodes, quadify);
    }

    generator.pop_node();
}

pub fn generate_divs(generator: &mut Generator, gltf: &gltf::Document, buffers: &Vec<buffer::Data>, desired_nodes: &Vec<String>, quadify: bool) {

    // TODO: this should only scale the base div, and not the geometry underneath
    let scale = 1.0;
    generator.push_node(Node{
        name: "base".to_string(),
        transforms: Transforms { translation: [0.0, 0.0, 0.0], rotation: [0.0, 0.0, 0.0, 1.0], scale: [scale, scale, scale] },
    });
    for scene in gltf.scenes() {
        for node in scene.nodes() {
            visit_children(generator, &node, &buffers, desired_nodes, quadify);
        }
    }
    generator.pop_node();

    println!("{}", &generator.output);
}

struct ImportedGltf {
    document: Document,
    buffers: Vec<buffer::Data>,
    _images: Vec<image::Data>,
}

#[wasm_bindgen]
pub struct DivGenerator {
    gltf: Result<ImportedGltf, String>,
}

#[wasm_bindgen]
impl DivGenerator {
    #[wasm_bindgen(constructor)]
    pub fn new(gltf_contents: &[u8]) -> Self {

        Self {
            gltf: gltf::import_slice(gltf_contents).map(|import| {
                ImportedGltf {
                    document: import.0,
                    buffers: import.1,
                    _images: import.2,
                }
            }).map_err(|e| e.to_string())
        }
    }

    pub fn import_error(&self) -> Option<String> {
        self.gltf.as_ref().err().cloned()
    }

    pub fn generate_divs(&self, scale: f32) -> String {
        if let Ok(gltf) = &self.gltf {
            let mut generator = Generator::new(scale);
            generate_divs(&mut generator, &gltf.document, &gltf.buffers, &Vec::new(), true);
            generator.output().to_string()
        } else {
            String::new()
        }
    }
}
