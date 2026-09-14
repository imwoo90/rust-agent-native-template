// cargo-generate post-expansion hook (Rhai)
// Automatically re-namespaces the project using scripts/init.sh upon generation.

let project_name = "";
if variable::is_set("project-name") {
    project_name = variable::get("project-name");
}

let agent = "all";
if variable::is_set("agent") {
    agent = variable::get("agent");
}

let clean = false;
if variable::is_set("clean") {
    clean = variable::get("clean");
}

if project_name != () && project_name != "" {
    let args = ["scripts/init.sh", project_name, "--agent", agent];
    if clean == true {
        args.push("--clean");
    }
    system::command("bash", args);
}
