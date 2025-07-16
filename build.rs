fn main() {
    println!("cargo:rustc-link-search=native=libs");
    println!("cargo:rustc-link-libs=static=jnihook");
}