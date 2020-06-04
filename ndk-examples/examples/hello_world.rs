#[cfg_attr(target_os = "android", ndk_glue::main(backtrace))]
fn main() {
    println!("hello world");
}
