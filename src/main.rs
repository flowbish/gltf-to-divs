use clap::{Parser, Subcommand};
use rendering::generate_divs;


#[derive(Subcommand, Debug)]
enum Command {
    GenerateDivs {
        #[arg(long)]
        scale: Option<f32>,
        #[arg(long)]
        quadify: bool,
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
        Command::GenerateDivs { scale, quadify } => {
            let mut generator = Generator::new(scale.unwrap_or(1.0));
            generate_divs(&mut generator, &gltf, &buffers, &args.nodes, quadify);
            println!("{}", generator.output());
        }
    }
}
