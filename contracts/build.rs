use foundry_compilers::{Project, ProjectPathsConfig};

fn main() {
    // configure the project with all its paths, solc, cache etc.
    let project_paths = ProjectPathsConfig::builder().build_with_root("foundry");
    let project = Project::builder()
        .paths(project_paths)
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
