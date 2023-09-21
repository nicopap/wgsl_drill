# Drill Naga Oil WGSL bindings

Analyze `#define` usages in your WGSL files and print bindings conditionally
enabled on `#ifdef FOO`.

## How to use

Example:

```sh
wgsl_drill -s prepass.wgsl -d SKINNED path/to/bevy/repo
```

<details><summary><h3>Sample outputs</h3></summary>

```sh
$ cargo run  -- -d TONEMAP -s sprite.wgsl bevy/
group(0) binding(0) var<uniform> view: View;
group(1) binding(0) var sprite_texture: texture_2d<f32>;
group(1) binding(1) var sprite_sampler: sampler;

$ cargo run  -- --list-defines -s sprite.wgsl bevy/
[
    "0",
    "TONEMAPPING_PASS",
    "TONEMAP_IN_SHADER",
    "TONEMAP_METHOD_ACES_FITTED",
    "TONEMAP_METHOD_AGX",
    "TONEMAP_METHOD_BLENDER_FILMIC",
# … …

$ cargo run  -- -d TONEMAPPING_PASS -d TONEMAP_IN_SHADER -s sprite.wgsl bevy/
group(0) binding(0) var<uniform> view: View;
group(0) binding(3) var dt_lut_texture: texture_3d<f32>;
group(0) binding(4) var dt_lut_sampler: sampler;
group(1) binding(0) var sprite_texture: texture_2d<f32>;
group(1) binding(1) var sprite_sampler: sampler;
```

</details>

Full description

```
Usage: wgsl_drill [OPTIONS] [SOURCE_DIRECTORIES]...

Arguments:
 [SOURCE_DIRECTORIES]...
     A list of directories to recursively walk to find wgsl source files

Options:
 -s, --shaders <SHADERS>
     The shader files for which to print the bindings.
     
     Accepts one or more shaders. If none provided, display a list.

 -d, --defines <DEFINES>
     The `#define`s to enable. To display a list, use --list-defines

 -l, --list-defines
     Display a list of all possible defines accessible from provided shaders

 -h, --help
     Print help (see a summary with '-h')
```


## How it works

As said earlier, it parses only **a subset** of the naga-oil WGSL dialect.
Notably:

- The `#ifdef`, `#else`, `#define_import_path`, `#import` CPP-like (C pre-processor)
  statements
- The WGSL line & block comments
- top-level resource variable declaration with a `group(x) binding(y)` attributes

We construct a rudimentary AST of all of those, and go through it, checking if
defines are available etc. We also read imports inline and conditionally.

## License

The code is licensed under MPL 2.0, see the license file for details.
