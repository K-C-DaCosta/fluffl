#!/bin/bash 

#build and put all scripts and binaries in one spot
wasm-pack build --debug  -t web ;
cp -r ./pkg/* ./out ;
cp ./templates/*.html ./out ; 

#start webserver at that spot
cd ./out; 
python3 -m http.server --bind 127.0.0.1 8080 ;
cd ..