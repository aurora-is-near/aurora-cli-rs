fn main() {
    shadow_rs::ShadowBuilder::builder()
        .build_pattern(shadow_rs::BuildPattern::RealTime)
        .build()
        .unwrap();
}
