fn android_sth() {
    println!("cargo:rustc-link-lib=c++_shared");
}

fn main() {
    #[cfg(target_os = "android")]
    {
        android_sth();
    }
}