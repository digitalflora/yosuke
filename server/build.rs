fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../yosuke.ico");
        res.compile().unwrap();
    }
    #[cfg(not(windows))]
    {}
}
