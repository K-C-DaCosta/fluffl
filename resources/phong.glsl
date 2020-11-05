#ifndef HEADER
#version 300 es
precision mediump float;
#endif

#ifndef UNIFORMS
uniform sampler2D page;
uniform vec4 text_color;
uniform mat4 projtion; 
uniform mat4 model; 
uniform mat4 scale;
#endif

#ifndef VERTEX_ATTRIBUTES
layout(location = 1) in vec4 vert_in; 
layout(location = 2) in vec2 uv_in; 
#endif

#ifndef VERTEX_SHADER
out vec2 tex_coord; 
void main(){
    tex_coord = uv_in;      
    gl_Position = vert_in; 
}
#endif

#ifndef FRAGMENT_SHADER
in vec2 tex_coord; 
out vec4 color; 
void main(){
    vec4 page = texture2D(page,tex_coord);
    float dist = page.w; 
    vec2 grad = vec2( dFdx(dist), dFdy(dist));  
    float grad_mag = length(grad)*1.5;
    color = vec4(1.)*smoothstep(0.5-grad_mag,0.5,dist);
}
#endif