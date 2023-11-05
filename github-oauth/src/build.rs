pub fn main() {
    println!("cargo:rerun-if-env-changed=CLIENT_ID");
    println!("cargo:rerun-if-env-changed=CLIENT_SECRET");
    println!("cargo:rerun-if-env-changed=AUTH_CALLBACK_URL");
}