use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::{fmt, fs, slice};

use anyhow::Result;
use walkdir::WalkDir;
use winnow::Parser;

use crate::ast::{Binding, IfDef, Resource, WgslFile};
use crate::parser::wgsl_file;

#[derive(Default)]
pub struct WgslFiles {
    files: HashMap<PathBuf, String>,
}
pub struct ParsedWgslFiles<'a> {
    file_names: HashMap<Cow<'a, str>, &'a Path>,
    import_paths: HashMap<&'a str, &'a Path>,
    files: HashMap<&'a Path, WgslFile<'a>>,
}
impl WgslFiles {
    pub fn load_all(&mut self, directory_path: &Path) -> Result<()> {
        for entry in WalkDir::new(directory_path) {
            let entry = entry?.into_path();
            if entry.extension() == Some("wgsl".as_ref()) {
                self.load(entry)?;
            }
        }
        Ok(())
    }
    #[allow(clippy::map_entry)] // false positive: using `entry` prevents us
                                // from using `path` to read from file system afterward
    fn load(&mut self, path: PathBuf) -> Result<()> {
        if !self.files.contains_key(&path) {
            let file_content = fs::read_to_string(&path)?;
            self.files.insert(path, file_content);
        }
        Ok(())
    }
    pub fn parse_all(&self) -> ParsedWgslFiles {
        let files = self.files.iter().map(|(path, content)| {
            let parsed = wgsl_file.parse(content).unwrap();
            (path.as_path(), parsed)
        });
        let files = files.collect::<HashMap<_, _>>();
        let import_paths = files
            .iter()
            .filter_map(|(path, file)| file.pub_name.map(|n| (n, *path)))
            .collect();
        let file_names = files
            .keys()
            .filter_map(|path| path.file_name().map(|f| (f.to_string_lossy(), *path)))
            .collect();
        ParsedWgslFiles { files, import_paths, file_names }
    }
}

struct DefList<'a, 'b> {
    def_list: &'b mut HashSet<String>,
    files: &'b ParsedWgslFiles<'a>,
}
impl<'a, 'b> DefList<'a, 'b> {
    fn bindings_for(&mut self, import_name: &str) -> Vec<Binding<'a>> {
        self.files.bindings_for(import_name, self.def_list)
    }
}
impl<'a> ParsedWgslFiles<'a> {
    pub fn defines(&self, file_paths: &[String]) -> Vec<&'a str> {
        let mut defines = Vec::new();
        let mut invalid_path = file_paths.is_empty();
        for path in file_paths {
            let Some(fs_path) = self.file_names.get(path.as_str()) else {
                invalid_path = true;
                println!("Invalid file: {path}");
                continue;
            };
            let file = self.files.get(fs_path).unwrap();
            defines.extend(file.resources.iter().flat_map(|r| r.defines(self)));
        }
        if invalid_path {
            println!("Valid paths are: {:#?}", self.file_names.keys());
        }
        defines.sort_unstable();
        defines.dedup();
        defines
    }
    fn defines_named(&self, import_path: &'a str) -> Vec<&'a str> {
        let fs_path = self.import_paths.get(import_path).unwrap();
        let file = self.files.get(fs_path).unwrap();
        file.resources
            .iter()
            .flat_map(|r| r.defines(self))
            .collect()
    }
    pub fn bindings(&self, def_list: &mut HashSet<String>, file_paths: &[String]) -> Bindings<'a> {
        let mut def_list = DefList { def_list, files: self };
        let mut bindings = Vec::new();
        let mut invalid_path = file_paths.is_empty();
        for path in file_paths {
            let Some(file) = self.file_names.get(path.as_str()) else {
                invalid_path = true;
                println!("Invalid file: {path}");
                continue;
            };
            bindings.extend(self.files.get(file).unwrap().bindings_for(&mut def_list));
        }
        if invalid_path {
            println!("Valid paths are: {:#?}", self.file_names.keys());
        }
        bindings.sort_unstable();
        bindings.dedup();
        Bindings { bindings }
    }
    fn bindings_for<'b>(
        &'b self,
        // declared shader name
        import_path: &str,
        def_list: &'b mut HashSet<String>,
    ) -> Vec<Binding<'a>> {
        let mut def_list = DefList { def_list, files: self };
        let fs_path = self.import_paths.get(import_path).unwrap();
        self.files.get(fs_path).unwrap().bindings_for(&mut def_list)
    }
}

impl<'a> WgslFile<'a> {
    fn bindings_for<'b>(&'b self, def_list: &mut DefList<'a, 'b>) -> Vec<Binding<'a>> {
        self.resources
            .iter()
            .flat_map(|r| r.bindings_for(def_list))
            .collect()
    }
}
impl<'a> IfDef<'a> {
    fn defines(&self, files: &ParsedWgslFiles<'a>) -> Vec<&'a str> {
        let then_branch = self.resources.iter().flat_map(|r| r.defines(files));
        let opt_else = self.else_branch.iter().flatten();
        let else_branch = opt_else.flat_map(|r| r.defines(files));
        let this = std::iter::once(self.name);
        this.chain(then_branch).chain(else_branch).collect()
    }
    fn bindings_for<'b>(&'b self, def_list: &mut DefList<'a, 'b>) -> Vec<Binding<'a>> {
        match &self.else_branch {
            _ if def_list.def_list.contains(self.name) => self.resources.iter(),
            Some(else_branch) => else_branch.iter(),
            None => slice::Iter::default(),
        }
        .flat_map(|r| r.bindings_for(def_list))
        .collect()
    }
}
impl<'a> Resource<'a> {
    fn defines(&self, files: &ParsedWgslFiles<'a>) -> Vec<&'a str> {
        match self {
            Resource::IfDef(ifdef) => ifdef.defines(files),
            Resource::Binding(_) => vec![],
            Resource::OilImport(import_name) => files.defines_named(import_name),
            Resource::Def(_) => vec![],
        }
    }
    fn bindings_for<'b>(&'b self, def_list: &mut DefList<'a, 'b>) -> Vec<Binding<'a>> {
        match self {
            Resource::IfDef(ifdef) => ifdef.bindings_for(def_list),
            Resource::Binding(b) => vec![*b],
            Resource::OilImport(import_name) => def_list.bindings_for(import_name),
            Resource::Def(define) => {
                def_list.def_list.insert(define.to_string());
                vec![]
            }
        }
    }
}
pub struct Bindings<'a> {
    bindings: Vec<Binding<'a>>,
}
impl fmt::Display for Bindings<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.bindings {
            writeln!(f, "group({}) binding({}) {};", b.group, b.binding, b.decl)?;
        }
        Ok(())
    }
}
