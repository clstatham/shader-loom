@vertex
fn vs_main(pos: vec3<f32>) -> @location(0) vec3<f32> {
    return pos + vec3<f32>(4.0, 5.0, 6.0);
}