fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "demo.gmproj".into());
    let project = demo_gen::build_demo();
    match gmformat::save(&project, std::path::Path::new(&path)) {
        Ok(()) => println!("demo project written to {}", path),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
