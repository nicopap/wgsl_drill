// ```ungrammar
// WgslFile = (OilImport)* Resource*
// Ifdef = '#ifdef' 'ident' Resource* IfdefEnd
// OilImport
//    = '#define_import_path' 'ident'
//    | '#import' 'ident' 'ident' (',' 'ident')*
//    | '#import' 'ident' 'as' 'ident'
// Resource
//    = Ifdef
//    | Binding
// IfdefEnd
//    = '#endif'
//    | '#else' Resource* '#endif'
// Binding =
//   '@group(' 'digits' ')'
//   '@binding(' 'digits' ')'
//   'var' ('<' Storage '>')?
//   'ident' ':' WgslType ';'
// WgslType = 'ident' ('<' 'ident' (',' WgslTypeParamTwo)? '>')?
// WgslTypeParamTwo = '#{PER_OBJECT_BUFFER_BATCH_SIZE}u' | 'write'
// Storage = 'uniform' | 'storage'
// ```
// use winnow::

fn main() {
    println!("Hello, world!");
}
