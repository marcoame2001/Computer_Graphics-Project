#version 430 core

in vec4 inColor;
in vec3 inNormal;
in vec3 inPos;
out vec4 color;
vec3 lightDir;

void main()
{   
    lightDir = normalize(vec3(0.8, -0.5, 0.9));
    float diff = max(0.0, dot(inNormal, -lightDir));
    color = vec4(inColor[0]*diff, inColor[1]*diff,inColor[2]*diff, inColor[3]); 

}
 