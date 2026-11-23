use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gltf::buffer;
use gltf::image;
use gltf::Material;
use gltf::image::Source;
use gltf::Document;

use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeshTransparency {
    /// This model is opaque, so backface culling optimizations may be applied.
    Opaque,
    /// This model or its texture is transparent and requires visibilty of the backs of polygons.
    /// This must be enabled for blockbench models.
    Transparent,
}

#[derive(Debug, Clone)]
struct Texture {
    uri: Option<String>,
}

struct GeneratorOutputWalker {
    indentation: usize,
    output: String,
}

impl GeneratorOutputWalker {
    fn new() -> Self {
        Self {
            indentation: 0,
            output: String::new(),
        }
    }

    fn add_output(&mut self, output: &str) {
        for line in output.lines() {
            for _ in 0..self.indentation {
                self.output.push_str("  ");
            }
            self.output.push_str(line);
            self.output.push('\n');
        }
    }

    pub fn output(&self) -> &str {
        &self.output
    }

    fn visit_node_start(&mut self, node: &Node, textures_to_output: &Vec<usize>, textures: &HashMap<usize, Texture>, scale: Option<f32>) {
        let name = &node.name;

        self.add_output(&format!(r#"<div class="node {name}" style=""#));

        for texture_index in textures_to_output {
            let texture_image = textures.get(&texture_index).and_then(
                |texture| texture.uri.clone()
            );
            if let Some(uri) = texture_image {
                self.add_output(&format!("  --texture-{texture_index}: url({uri});"));
            }
        }
        
        // base scale
        if let Some(scale) = scale {
            self.add_output(&format!("  --base-scale: {scale};"));
        }

        // translation
        let [x, y, z] = node.transforms.translation;
        if x != 0.0 {
            self.add_output(&format!("  --node-inherent-translate-x: {x}"));
        }
        if y != 0.0 {
            self.add_output(&format!("  --node-inherent-translate-y: {y}"));
        }
        if z != 0.0 {
            self.add_output(&format!("  --node-inherent-translate-z: {z}"));
        }

        let [x, y, z, w] = node.transforms.rotation;
        let y = -1.0 * y;
        if x != 0.0 {
            self.add_output(&format!("  --node-inherent-rotation-x: {x}"));
        }
        if y != 0.0 {
            self.add_output(&format!("  --node-inherent-rotation-y: {y}"));
        }
        if z != 0.0 {
            self.add_output(&format!("  --node-inherent-rotation-z: {z}"));
        }        
        let angle = -2.0 * w.acos();
        if angle != 0.0 {
            self.add_output(&format!("  --node-inherent-rotation-angle: {w}"));
        }

        let [x, y, z] = node.transforms.scale;
        if x != 1.0 {
            self.add_output(&format!("  --node-inherent-scale-x: {x}"));
        }
        if y != 1.0 {
            self.add_output(&format!("  --node-inherent-scale-y: {y}"));
        }
        if z != 1.0 {
            self.add_output(&format!("  --node-inherent-scale-z: {z}"));
        }

        self.add_output("\">");
        
        self.indentation += 1;
    }

    fn visit_node_end(&mut self) {
        self.indentation -= 1;
        self.add_output("</div>");
    }

    fn visit_face(&mut self, face: &Face, class: &str, mesh_transparency: MeshTransparency, reverse_orientation: bool) {
        let mut classes = vec![class];
        if mesh_transparency == MeshTransparency::Opaque {
            classes.push("opaque");
        }
        let classes = classes.join(" ");

        // TODO: add an argument to reverse b and c.
        let a = face.a;
        let b = if reverse_orientation { face.c } else { face.b };
        let c = if reverse_orientation { face.b } else { face.c };

        let a_x = a.position[0];
        let a_y = a.position[1] * -1.0;
        let a_z = a.position[2];
        let b_x = b.position[0];
        let b_y = b.position[1] * -1.0;
        let b_z = b.position[2];
        let c_x = c.position[0];
        let c_y = c.position[1] * -1.0;
        let c_z = c.position[2];
        let color = face.color.to_string();
        let texture_var = face.texture_index.map(
            |idx| format!("--texture-{idx}")
        );
        let tex_a_s = a.texture[0];
        let tex_a_t = a.texture[1];
        let tex_b_s = b.texture[0];
        let tex_b_t = b.texture[1];
        let tex_c_s = c.texture[0];
        let tex_c_t = c.texture[1];
        self.add_output(&format!(r#"<div class="{classes}" style=""#));
        self.add_output(&format!("--a-x: calc(var(--base-scale) * {a_x});"));
        self.add_output(&format!("--a-y: calc(var(--base-scale) * {a_y});"));
        self.add_output(&format!("--a-z: calc(var(--base-scale) * {a_z});"));
        self.add_output(&format!("--b-x: calc(var(--base-scale) * {b_x});"));
        self.add_output(&format!("--b-y: calc(var(--base-scale) * {b_y});"));
        self.add_output(&format!("--b-z: calc(var(--base-scale) * {b_z});"));
        self.add_output(&format!("--c-x: calc(var(--base-scale) * {c_x});"));
        self.add_output(&format!("--c-y: calc(var(--base-scale) * {c_y});"));
        self.add_output(&format!("--c-z: calc(var(--base-scale) * {c_z});"));
        if let Some(texture_var) = texture_var {
            self.add_output(&format!("--texture-image: var({texture_var});"));
        } else {
            self.add_output(&format!("--texture-background-color: {color};"));
        }
        self.add_output(&format!("--texture-a-s: {tex_a_s};"));
        self.add_output(&format!("--texture-a-t: {tex_a_t};"));
        self.add_output(&format!("--texture-b-s: {tex_b_s};"));
        self.add_output(&format!("--texture-b-t: {tex_b_t};"));
        self.add_output(&format!("--texture-c-s: {tex_c_s};"));
        self.add_output(&format!("--texture-c-t: {tex_c_t};"));
        self.add_output(r#""></div>"#);
    }

}

#[derive(Debug, Clone)]
enum GeneratorChild {
    Node(Rc<RefCell<GeneratorNode>>),
    Face(GeneratorFace),
}

#[derive(Debug, Clone)]
struct GeneratorFace {
    face: Face,
    class: &'static str,
}

impl GeneratorFace {
    fn new(face: Face, class: &'static str) -> Self {
        Self { 
            face,
            class,
        }
    }
}

#[derive(Debug, Clone)]
struct GeneratorNode {
    node: Node,
    textures_to_output: Vec<usize>,
    scaling_factor: Option<f32>,
    children: Vec<GeneratorChild>,
}

impl GeneratorNode {
    fn new(node: Node) -> Self {
        Self { 
            node, 
            textures_to_output: Vec::new(),
            scaling_factor: None,
            children: Vec::new(),
        }
    }

    fn new_base(scaling_factor: f32) -> Self {
        Self { 
            node: Node::base(), 
            textures_to_output: Vec::new(),
            scaling_factor: Some(scaling_factor),
            children: Vec::new(),
        }
    }
}

pub struct Generator {
    textures: HashMap<usize, Texture>,
    base: Rc<RefCell<GeneratorNode>>,
    node_stack: Vec<Rc<RefCell<GeneratorNode>>>,
    mesh_transparency: MeshTransparency,
    reverse_orientation: bool
}

impl Generator {
    pub fn new(scale: f32, mesh_transparency: MeshTransparency, reverse_orientation: bool) -> Self {
        let base = Rc::new(RefCell::new(GeneratorNode::new_base(scale)));
        Self {
            textures: HashMap::new(),
            base: base.clone(),
            node_stack: vec![base.clone()],
            mesh_transparency,
            reverse_orientation,
        }
    }

    fn output_recursive(&self, output: &mut GeneratorOutputWalker, node: &GeneratorNode) {
        output.visit_node_start(&node.node, &node.textures_to_output, &self.textures, node.scaling_factor);
        for child in &node.children {
            match child {
                GeneratorChild::Node(node) => self.output_recursive(output, &*node.borrow()),
                GeneratorChild::Face(face) => output.visit_face(&face.face, face.class, self.mesh_transparency, self.reverse_orientation),
            }
        }
        output.visit_node_end();
    }

    pub fn output(&self) -> String {
        let mut output = GeneratorOutputWalker::new();
        self.output_recursive(&mut output, &*self.base.borrow());
        output.output().to_string()
    }

    fn visit_texture(&mut self, index: usize, texture: Texture) {
        self.textures.insert(index, texture);
        let top = self.node_stack.last().unwrap();
        top.borrow_mut().textures_to_output.push(index);
    }

    fn push_node(&mut self, node: Node) {
        let top = self.node_stack.last().unwrap();
        let node = Rc::new(RefCell::new(GeneratorNode::new(node)));
        top.borrow_mut().children.push(GeneratorChild::Node(node.clone()));
        self.node_stack.push(node);
    }

    fn pop_node(&mut self) {
        self.node_stack.pop();
    }

    fn visit_face(&mut self, face: Face, class: &'static str) {
        let top = self.node_stack.last().unwrap();
        top.borrow_mut().children.push(GeneratorChild::Face(GeneratorFace::new(face, class)));
    }

    fn visit_rectangle(&mut self,  rectangle: &Rectangle) {
        self.visit_face(rectangle.face.clone(), "rect")
    }

    fn visit_triangle(&mut self, triangle: &Triangle) {
        self.visit_face(triangle.face.clone(), "tri")

    }
}

#[derive(Debug, Clone)]
struct Transforms {
    translation: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
}

#[derive(Debug, Clone)]
struct Node {
    name: String,
    transforms: Transforms,
}

impl Node {
    fn base() -> Self {
        Self {
            name: "base".to_string(),
            transforms: Transforms { translation: [0.0, 0.0, 0.0], rotation: [0.0, 0.0, 0.0, 1.0], scale: [1.0, 1.0, 1.0] },
        }
    }
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
    texture_index: Option<usize>,
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

    let texture_index = material.pbr_metallic_roughness().base_color_texture().and_then(
        |info| Some(info.texture().index())
    );

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
                    texture_index,
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

fn visit_children<'a>(generator: &mut Generator, node: &gltf::Node<'a>, buffers: &Vec<buffer::Data>, desired_nodes: &Vec<String>, exclude_nodes: &Vec<String>, quadify: bool) {
    let name = node.name().unwrap_or("N/A");

    if !desired_nodes.is_empty() && !desired_nodes.contains(&name.to_string()) || exclude_nodes.contains(&name.to_string()) {
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
        visit_children(generator, &child, buffers, desired_nodes, exclude_nodes, quadify);
    }

    generator.pop_node();
}

pub fn generate_divs(generator: &mut Generator, gltf: &gltf::Document, buffers: &Vec<buffer::Data>, desired_nodes: &Vec<String>, exclude_nodes: &Vec<String>, quadify: bool) {
    for texture in gltf.textures() {
        let uri = texture.source().and_then(
            |image| {
                match image.source() {
                    Source::Uri { uri, .. } => Some(uri.to_string()),
                    _ => None,
                }
            }
        );
        generator.visit_texture(texture.index(), Texture { uri });
    }

    for scene in gltf.scenes() {
        for node in scene.nodes() {
            visit_children(generator, &node, &buffers, desired_nodes, exclude_nodes, quadify);
        }
    }
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
            let mut generator = Generator::new(scale, MeshTransparency::Transparent, false /* reverse_orientation */);
            generate_divs(&mut generator, &gltf.document, &gltf.buffers, &Vec::new(), &Vec::new(), true /* quadify */);
            generator.output().to_string()
        } else {
            String::new()
        }
    }
}
