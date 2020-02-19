#[cfg(target_os = "android")]
ndk_glue::ndk_glue!(main);

fn main() {
    println!("hello world");
}
