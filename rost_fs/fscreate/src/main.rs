use std::env;
use std::path::Path;

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() < 4 {
        println!("Usage: fscreate [image location] [image size in blocks] [root folder path]");
        std::process::exit(-1);
    }

    let size: usize = args[2].parse().expect("image size is not a number");

    let fraction = fscreate::create_image(Path::new(&args[1]), size, Path::new(&args[3]))
        .expect("image creation failed");

    let total_size = size as f64 * 4096.;
    println!(
        "Used ~{}% of image. ({}/{} MiB)",
        fraction * 100.,
        total_size * fraction / 1000000.,
        total_size / 1000000.
    );
}
