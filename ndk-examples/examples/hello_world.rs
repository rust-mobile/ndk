#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
fn main() {
    println!("hello world");
}
