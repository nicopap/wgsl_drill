// ```ungrammar
// WgslFile = (ImportDefine)? Resource*
// Ifdef = '#ifdef' 'ident' Resource* IfdefEnd
// ImportDefine = '#define_import_path' 'ident'
// OilImport
//    = '#import' 'ident' ('ident' (',' 'ident')*)?
//    | '#import' 'ident' 'as' 'ident'
// Resource
//    = Ifdef
//    | Binding
//    | OilImport
//    | Define
// Define = '#define' 'ident'
// IfdefEnd
//    = '#endif'
//    | '#else ifdef' 'ident' Resource* '#endif'
//    | '#else' Resource* '#endif'
// Binding =
//   '@group(' 'digits' ')'
//   '@binding(' 'digits' ')'
//   'var' ('<' Storage '>')?
//   'ident' ':' WgslType ';'
// WgslType = 'ident' ('<' 'ident' (',' WgslTypeParamTwo)? '>')?
// WgslTypeParamTwo = '#{' 'ident' '}' ('u')? | 'write'
// Storage = 'uniform' | 'storage'
// ```
use winnow::ascii::{digit1, multispace0 as ws};
use winnow::combinator::{
    alt, delimited, fail, not, opt, preceded, repeat, separated0, success, terminated,
};
use winnow::error::StrContext;
use winnow::stream::AsChar;
use winnow::token::{take_till0, take_till1, take_until0, take_while};
use winnow::trace::trace;
use winnow::{dispatch, prelude::*};

use crate::ast::*;

macro_rules! ctx {
    ($inner:expr) => {
        $inner.context(StrContext::Label(stringify!($inner)))
    };
}
macro_rules! t {
    ($inner:expr) => {
        trace(stringify!($inner), $inner)
    };
}
macro_rules! ws {
    ($inner:expr) => {
        delimited(ws, $inner, ws)
    };
}

fn ident<'a>(input: &mut &'a str) -> PResult<&'a str> {
    let ident = (take_while(1.., |c: char| {
        c.is_alphanum() || ['#', '@', '_', ':'].contains(&c)
    }),)
        .recognize();

    t!(ident).parse_next(input)
}
fn peol_comment(i: &mut &str) -> PResult<()> {
    let peol_comment = ("//", take_till1(['\n', '\r']));
    t!(peol_comment).void().parse_next(i)
}
fn pinline_comment(i: &mut &str) -> PResult<()> {
    let inline_comment = ("/*", take_until0("*/"), "*/");
    t!(inline_comment).void().parse_next(i)
}
/// The rest of the WGSL! file, all but the naga_oil CPP symbols and binding
/// declarations.
fn ignore(input: &mut &str) -> PResult<()> {
    let most_code = || take_till0(['/', '#', '@']);
    let non_group = ("@", not("group")).void();
    let irrelevant = separated0(
        most_code(),
        alt((
            non_group,
            peol_comment,
            pinline_comment,
            "/".void(),
            "#{".void(),
            "#SLICE_COUNT".void(),
            "#SAMPLES_PER_SLICE_SIDE".void(),
        )),
    );
    t!(irrelevant).parse_next(input)
}

pub fn wgsl_file<'a>(input: &mut &'a str) -> PResult<WgslFile<'a>> {
    (
        preceded(ignore, opt(terminated(import_define, ignore))),
        repeat(.., resource),
    )
        .map(|(pub_name, resources)| WgslFile { pub_name, resources })
        .parse_next(input)
}

fn import_define<'a>(input: &mut &'a str) -> PResult<&'a str> {
    delimited(("#define_import_path", ws), ident, "\n").parse_next(input)
}

fn oil_import<'a>(input: &mut &'a str) -> PResult<&'a str> {
    let sep = separated0::<_, _, (), _, _, _, _>;
    let as_import = (ws!("as"), ident).void();
    let multi_import = preceded(repeat::<_, _, (), _, _>(.., " "), sep(ident, ", "));
    let import = delimited(ws, ident, (alt((as_import, multi_import)), "\n"));
    t!(import).parse_next(input)
}

fn if_def<'a>(input: &mut &'a str) -> PResult<IfDef<'a>> {
    (
        delimited(ws, ident, ignore),
        repeat(.., resource),
        if_def_end,
    )
        .map(|(name, resources, else_branch)| IfDef { name, resources, else_branch })
        .parse_next(input)
}

fn if_not_def<'a>(input: &mut &'a str) -> PResult<IfDef<'a>> {
    (
        delimited(ws, ident, ignore),
        repeat(.., resource),
        if_def_end,
    )
        .map(|(name, resources, else_branch)| IfDef {
            name,
            resources: else_branch.unwrap_or(vec![]),
            else_branch: Some(resources),
        })
        .parse_next(input)
}

fn if_cond<'a>(input: &mut &'a str) -> PResult<IfDef<'a>> {
    (
        delimited(ws, ident, ignore),
        repeat(.., resource),
        if_def_end,
    )
        .map(|(name, resources, else_branch)| IfDef { name, resources, else_branch })
        .parse_next(input)
}
fn define<'a>(input: &mut &'a str) -> PResult<&'a str> {
    preceded(ws, ident).parse_next(input)
}
fn resource<'a>(input: &mut &'a str) -> PResult<Resource<'a>> {
    let mut resource = dispatch! { ident ;
        "@group" => terminated(ctx!(binding), ignore).map(Resource::Binding),
        "#define" => terminated(ctx!(define), ignore).map(Resource::Def),
        "#ifdef" => terminated(ctx!(if_def), ignore).map(Resource::IfDef),
        "#ifndef" => terminated(ctx!(if_not_def), ignore).map(Resource::IfDef),
        "#if" => terminated(ctx!(if_cond), ignore).map(Resource::IfDef),
        "#import" => terminated(ctx!(oil_import), ignore).map(Resource::OilImport),
        _ => fail,
    };
    resource.parse_next(input)
}

fn else_if<'a>(input: &mut &'a str) -> PResult<Vec<Resource<'a>>> {
    alt((
        preceded(
            " ifdef ",
            terminated(ctx!(if_def), ignore).map(|d| vec![Resource::IfDef(d)]),
        ),
        delimited(ignore, repeat(.., resource), "#endif"),
    ))
    .parse_next(input)
}
fn if_def_end<'a>(input: &mut &'a str) -> PResult<Option<Vec<Resource<'a>>>> {
    let mut if_def_end = dispatch! { ident ;
        "#else" => else_if.map(Some),
        "#endif" => success(None),
        _ => fail,
    };
    if_def_end.parse_next(input)
}

fn binding<'a>(input: &mut &'a str) -> PResult<Binding<'a>> {
    (
        preceded("(", digit1).map(|i: &str| i.parse().unwrap()),
        delimited(") @binding(", digit1, (")", ws)).map(|i: &str| i.parse().unwrap()),
        t!(wgsl_declaration).recognize(),
    )
        .map(|(group, binding, decl)| Binding { decl, group, binding })
        .parse_next(input)
}

fn wgsl_type(input: &mut &str) -> PResult<()> {
    let wtype = (ident, opt(("<", ident, opt((", ", wgsl_param2)), ">")));
    t!(wtype).void().parse_next(input)
}

fn wgsl_param2(input: &mut &str) -> PResult<()> {
    let def_const = delimited("#{", ident, ("}", opt("u")));
    alt(("write", def_const)).void().parse_next(input)
}

fn var(input: &mut &str) -> PResult<()> {
    alt(("var<uniform>", "var<storage>", "var"))
        .void()
        .parse_next(input)
}

fn wgsl_declaration<'a>(input: &mut &'a str) -> PResult<&'a str> {
    let mut declaration = preceded(ws!(var), (ident, " ", wgsl_type)).recognize();
    declaration.parse_next(input)
}
