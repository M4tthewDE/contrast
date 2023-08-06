fn main() {
    glib_build_tools::compile_resources(
        &["composite_templates/1/resources"],
        "composite_templates/1/resources/resources.gresource.xml",
        "composite_templates_1.gresource",
    );
}
