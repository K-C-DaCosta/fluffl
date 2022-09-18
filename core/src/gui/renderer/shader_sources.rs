pub const ROUNDED_BOX_SHADER_SOURCE: &'static str = r"
    #ifndef HEADER
        #version 300 es
        precision mediump float;
        uniform float edge_thickness; 
        uniform vec4  edge_color; 
        uniform vec4  position;
        uniform vec4  bounds;
        uniform vec4  roundness; 
        uniform vec4  background_color;
        uniform mat4  modelview;
        uniform mat4  proj;  
    #endif

    #ifndef VERTEX_SHADER
        layout(location = 0) in vec4 attr_pos;
        out vec4 world_space_pos;
        void main(){
            vec4 world_space = modelview*attr_pos;
            world_space_pos = world_space;  
            //convert worldspace to NDC 
            gl_Position = proj*world_space;
        }
    #endif
    
    #ifndef FRAGMENT_SHADER
        in vec4 world_space_pos;
        out vec4 final_color; 

        float when_lt(float x, float y){
            return max(sign(y-x),0.0);
        }

        float linear_step(float e0,float e1,float x){
            return clamp((x - e0)/(e1-e0),0.0,1.0);
        }

        float sdRoundBox( in vec2 p, in vec2 b, in vec4 r ) 
        {            
            //sdf eval starts here 
            r.xy = (p.x>0.0)?r.xy : r.zw;
            r.x  = (p.y>0.0)?r.x  : r.y;
            vec2 q = abs(p)-b+r.x;
            return min(max(q.x,q.y),0.0) + length(max(q,0.0)) - r.x;
        }

        void main(){
            
            float max_iso_value = 0.0;
            float sband = 2.0;

            //use modelview matrix to compute width and height bounding box 
            //by using the fact that the geometry is ALWAYS a unit-square in the bottom-right quadrant 
            vec4 horizontal_disp = modelview*(vec4(1.,0.,0.,1.0) - vec4(0.,0.,0.,1.0));
            vec4 vertical_disp = modelview*(vec4(0.0,1.0,0.,1.0) - vec4(0.,0.,0.,1.0));
            float w = horizontal_disp.x; 
            float h = vertical_disp.y;

            vec4 pos = world_space_pos;
            float d = sdRoundBox(pos.xy - position.xy - bounds.xy*0.5,bounds.xy*0.5,roundness);
            float d_epsilon = length(vec2(dFdx(d),dFdy(d)));

            // max_iso_value = d_epsilon; 
            if (d > d_epsilon){
                discard; 
            }
            

            float w0 = smoothstep(max_iso_value-edge_thickness,max_iso_value-edge_thickness-sband,d);
            float w1 = smoothstep(max_iso_value+d_epsilon,max_iso_value-edge_thickness,d);
            // float w1 = linear_step(max_iso_value+d_epsilon,max_iso_value-edge_thickness,d);
            // w1 = 1.0 - pow(1.0 - w1,8.0);

            final_color = vec4(0);

            //main body
            final_color += background_color*w0;
            
            //edge
            float edge_enabled_mask = when_lt(0.01,edge_thickness);
            vec4  final_edge_color = edge_color*edge_enabled_mask + background_color*(1.0 - edge_enabled_mask);
            final_color += final_edge_color*w1 - final_edge_color*w0;
        }
    #endif
";

pub const RECTANGLE_SHADER_SOURCE: &'static str = r"
    #ifndef HEADER
        #version 300 es
        precision mediump float;
        uniform vec4 background_color;
        uniform mat4 modelview;
        uniform mat4 proj;  
    #endif

    #ifndef VERTEX_SHADER
        layout(location = 0) in vec4 attr_pos;
        out vec4 world_space_pos;
        void main(){
            vec4 world_space = modelview*attr_pos;
            world_space_pos = world_space;  
            //convert worldspace to NDC 
            gl_Position = proj*world_space;
        }
    #endif

    #ifndef FRAGMENT_SHADER
        in vec4 world_space_pos;
        out vec4 final_color; 
        void main(){
            final_color = background_color; 
        }
    #endif
";
