#version 430 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 vertexColor;
layout(location = 2) in vec3 normal;

out vec4 inColor;
out vec3 inNormal;
out vec3 inPos;
uniform mat4x4 modelMatrix;
uniform mat4x4 MVP;
vec4 origin;

void main()  
{

/*mat4 matrix1 = {
        {1, 0, 0, 0},
        {0, 1, 0, 0},
        {0, 0, 1, 0},
        {0, 0, 0, 1},
    };

  
float a = sin(time);
float b = 0.0;
float c = 0.0;
float d = 0.0;
float e = 1.0;
float f = 0.0;
mat4 matrix2 = {
        { a, b, 0, c},
        { d, e, 0, f},
        { 0, 0, 1, 0},
        { 0, 0, 0, 1},
    };
*/
    origin = vec4(position, 1.0);
    gl_Position = MVP*origin;
    inPos = vec3(modelMatrix * origin);
    inColor = vertexColor;
    inNormal = normalize(mat3(modelMatrix)*normal);
}

