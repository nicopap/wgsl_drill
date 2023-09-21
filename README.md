# Shader conditional binding

This parses a subset of naga-oil's WGSL to gather bindings used by various
shader combinations.

## How it works?

As said earlier, it parses only **a subset** of the naga-oil WGSL dialect.
Notably:

- The `#ifdef`, `#else`, `#define_import_path`, `#import` CPP-like (C pre-processor)
  statements
- The WGSL standard line & block comments
- top-level resource variable declaration with a `group(x) binding(y)` attributes

With this, it has a graph of define -> binding resource and can tell you what
a specific set of defines will requires as binding

## Grammar

```ungrammar
WgslFile = (OilImport)* Resource*
Ifdef = '#ifdef' 'ident' Resource* IfdefEnd
OilImport
   = '#define_import_path' 'ident'
   | '#import' 'ident' 'ident' (',' 'ident')*
   | '#import' 'ident' 'as' 'ident'
Resource
   = Ifdef
   | Binding
IfdefEnd
   = '#endif'
   | '#else' Resource* '#endif'
Binding =
  '@group(' 'digits' ')'
  '@binding(' 'digits' ')'
  'var' ('<' Storage '>')?
  'ident' ':' WgslType ';'
WgslType = 'ident' ('<' 'ident' (',' WgslTypeParamTwo)? '>')?
WgslTypeParamTwo = '#{PER_OBJECT_BUFFER_BATCH_SIZE}u' | 'write'
Storage = 'uniform' | 'storage'
```

