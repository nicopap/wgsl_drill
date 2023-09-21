#[derive(Debug)]
pub struct WgslFile<'a> {
    pub pub_name: Option<&'a str>,
    pub resources: Vec<Resource<'a>>,
}
#[derive(Clone, Debug)]
pub enum Resource<'a> {
    IfDef(IfDef<'a>),
    Binding(Binding<'a>),
    OilImport(&'a str),
    Def(&'a str),
}
#[derive(Clone, Debug)]
pub struct IfDef<'a> {
    pub name: &'a str,
    pub resources: Vec<Resource<'a>>,
    pub else_branch: Option<Vec<Resource<'a>>>,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Binding<'a> {
    pub group: u32,
    pub binding: u32,
    /// The whole var<uniform> fooBar: FooBaz
    pub decl: &'a str,
}
