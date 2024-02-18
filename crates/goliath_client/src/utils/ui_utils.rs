use eframe::egui;

pub fn generate_font_cache(cc: &eframe::CreationContext) {
    let mut font_definitions = egui::FontDefinitions::default();
    font_definitions.font_data.insert(
        "main".to_string(),
        egui::FontData::from_static(include_bytes!("../../resources/DIGITALDREAM.ttf")),
    );
    font_definitions.families.insert(
        egui::FontFamily::Name("main".into()),
        vec!["main".to_string()],
    );
    cc.egui_ctx.set_fonts(font_definitions);
}
