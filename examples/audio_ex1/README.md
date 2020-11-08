# Audio Ex 1 
A desktop example

# Build Instructions
## Desktop: 
Desktop is the default target (currently using sdl2) so just do a:
```
cargo build
```
and it should "just werk" assuming you have the sdl2 libs installed.
On windows you may have to place the sdl2.dll in a lib folder or something before you run the program 

## Browser:

```
cargo build --release --target=wasm32-unknown-unknown --no-default-features --features='web'
wasm-bindgen ./target/wasm32-unknown-unknown/release/audio_ex1.wasm --out-dir ../../wasm_bins/examples/audio_ex_1 --target web
```

# Run Instructions

## Desktop
simply just:
```
cargo run
```

## Browser:

Unfortunately you have to host the files in `./generated` on a webserver in order for the browser to run the wasm module. 
Something like `simple-http-server` should do for this task but you could use a python webserver, or your own custom server it really doesn't matter.  

If you want to use `simple-http-server` just do:
```
cargo install simple-http-server
```

to start the webserver just: 

```
simple-http-server --ip 127.0.0.1  -p 8080 ../../wasm_bins
```

Before opening the browser you have to place a simple .html document called: `index.html` in the `generated` directory.

`index.html` should look something like this: 
```
<html>
    <head>
        <meta content="text/html;charset=utf-8" http-equiv="Content-Type" />
    </head>
    <body>
        <canvas id="fluffl" width="800" height="600"></canvas>
        <script type="module">
            import init from "./audio_ex1.js";
            // import {set_xml_config} from './snippets/glue_core-513a7bde24bf152f/js/aux_util.js';
            // push_xml_config("<window><config_path>um hello based department</config_path></window>");
            async function run() {
                var demo = await init();
            }
            run();
        </script>
    </body>
</html>
```  

# On useage
`spacebar` and music should play.
`Page up` to increase volume
`Page down` to decrease volume
`R-key` to restart music