// cargo-generate post-expansion hook (Rhai)
// Automatically re-namespaces the project using scripts/init.sh upon generation.

let project_name = variable["project-name"];
if project_name != () && project_name != "" {
    system::command("bash", ["scripts/init.sh", project_name]);
}
