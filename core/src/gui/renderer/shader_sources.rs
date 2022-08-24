pub const FRAME_SHADER_SOURCE: &'static str = r"
    #ifndef HEADER
        #version 300 es
        precision mediump float;
        uniform vec4 edge_color; 
        uniform vec4 position;
        uniform vec4 bounds;
        uniform vec4 roundness; 
        uniform vec4 background_color;
        uniform vec4 null_color; 
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

        float sdRoundBox( in vec2 p, in vec2 b, in vec4 r ) 
        {            
            //sdf eval starts here 
            r.xy = (p.x>0.0)?r.xy : r.zw;
            r.x  = (p.y>0.0)?r.x  : r.y;
            vec2 q = abs(p)-b+r.x;
            return min(max(q.x,q.y),0.0) + length(max(q,0.0)) - r.x;
        }

        void main(){
            float max_depth = -5.0;
            float band = 3.0;

            //use modelview matrix to compute width and height bounding box 
            //by using the fact that the geometry is ALWAYS a unit-square in the bottom-right quadrant 
            vec4 horizontal_disp = modelview*(vec4(1.,0.,0.,1.0) - vec4(0.,0.,0.,1.0));
            vec4 vertical_disp = modelview*(vec4(0.0,1.0,0.,1.0) - vec4(0.,0.,0.,1.0));
            float w = horizontal_disp.x; 
            float h = vertical_disp.y;

            vec4 pos = world_space_pos;
            float d = sdRoundBox(pos.xy - position.xy - bounds.xy*0.5,bounds.xy*0.5,roundness);

            float d_epsilon = length(vec2(dFdx(d),dFdy(d)));
            

            float w0 = smoothstep(max_depth+band,max_depth,d);
            float w1 = smoothstep(d_epsilon,max_depth+band,d);
            
            final_color = vec4(0);

            //main body
            final_color += background_color*w0;
            
            //edge
            final_color += edge_color*w1 - edge_color*w0;
        }
    #endif
";