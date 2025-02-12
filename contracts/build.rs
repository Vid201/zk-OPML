use foundry_compilers::{
    artifacts::Settings, multi::MultiCompilerSettings, solc::SolcSettings, Project,
    ProjectPathsConfig,
};

fn main() {
    // configure the project with all its paths, solc, cache etc.
    let project_paths = ProjectPathsConfig::builder().build_with_root("foundry");
    let mut settings = MultiCompilerSettings::default();
    let solc_settings = SolcSettings {
        settings: Settings {
            via_ir: Some(true),
            ..Default::default()
        },
        ..Default::default()
    };
    settings.solc = solc_settings;
    let project = Project::builder()
        .paths(project_paths)
        .settings(settings)
        .build(Default::default())
        .expect("failed to build project");

    let output = project.compile().expect("failed to compile project");

    if output.has_compiler_errors() {
        panic!("{}", format!("{:?}", output.output().errors));
    }

    // Tell Cargo that if a source file changes, to rerun this build script.
    project.rerun_if_sources_changed();
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=test");
}
