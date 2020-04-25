fn main() {
    let lib_dir = "/Library/Frameworks/";
    println!("cargo:rustc-link-search=framework={}", lib_dir);
}
