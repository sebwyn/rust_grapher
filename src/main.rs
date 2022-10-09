use rust_grapher::app::App;

//built following https://sotrh.github.io/learn-wgpu/beginner/
fn main() {
    pollster::block_on(App::run());
}