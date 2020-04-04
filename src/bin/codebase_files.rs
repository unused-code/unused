use codebase_files::CodebaseFiles;

fn main() {
    let files = CodebaseFiles::all();
    for f in files.paths.iter() {
        println!("{}", f);
    }
}
