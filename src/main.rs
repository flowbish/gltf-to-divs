use clap::{Parser, Subcommand};
use rendering::{Generator, generate_divs};

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum MeshTransparency {
    /// This model is opaque, so backface culling optimizations may be applied.
    #[default]
    Opaque,
    /// This model or its texture is transparent and requires visibilty of the backs of polygons.
    /// This must be enabled for blockbench models.
    Transparent,
}

impl From<MeshTransparency> for rendering::MeshTransparency {
    fn from(mesh_transparency: MeshTransparency) -> Self {
        match mesh_transparency {
            MeshTransparency::Opaque => Self::Opaque,
            MeshTransparency::Transparent => Self::Transparent,
        }
    }
}


#[derive(Subcommand, Debug)]
enum Command {
    GenerateDivs {
        #[arg(long)]
        exclude_node: Vec<String>,
        #[arg(long)]
        scale: Option<f32>,
        #[arg(long)]
        quadify: bool,
        #[arg(long, help="Setting mesh transparency to transparent allow the back face of faces to show.")]
        mesh_transparency: MeshTransparency,
        #[arg(long, help="Reverse the orientation of faces. Necessary if your model has a y-negative winding (like blockbench models) and you choose an opaque mesh.")]
        reverse_orientation: bool,
    },
    ListNodes,
}

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Command,
    path: String,
    #[arg(long)]
    nodes: Vec<String>,
}


fn visit_list(node: &gltf::Node<'_>) {
    let name = node.name().unwrap_or("N/A");
    println!("{name}");
    for node in node.children() {
        visit_list(&node)
    }
}

fn list_nodes(gltf: &gltf::Document) {
    for scene in gltf.scenes() {
        for node in scene.nodes() {
            visit_list(&node);
        }
    }
}

fn main() {
    let args = Args::parse();

    let (gltf, buffers, _images) = gltf::import(&args.path).unwrap();


    match args.command {
        Command::ListNodes => {
            list_nodes(&gltf);
        }
        Command::GenerateDivs { exclude_node, scale, quadify, mesh_transparency, reverse_orientation } => {
            let mut generator = Generator::new(scale.unwrap_or(1.0), mesh_transparency.into(), reverse_orientation);
            generate_divs(&mut generator, &gltf, &buffers, &args.nodes, &exclude_node, quadify);
            println!("{}", generator.output());
        }
    }
}
