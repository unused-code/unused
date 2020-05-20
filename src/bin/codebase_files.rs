use codebase_files::CodebaseFiles;

fn main() {
    let files = CodebaseFiles::all();
    for f in files.paths.iter().filter_map(|v| v.to_str()) {
        println!("{}", f);
    }
}
