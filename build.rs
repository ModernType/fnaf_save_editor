
fn main() {
    slint_build::compile("ui/main.slint").unwrap();
    let mut res = winresource::WindowsResource::new();
    res.set_icon("icon.ico");
    res.compile().unwrap();
}