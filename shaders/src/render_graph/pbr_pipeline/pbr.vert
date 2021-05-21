#version 450

layout(location = 0) in vec3 Vertex_Position;
//layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec2 Vertex_Uv;
layout(location = 3) in int Per_Face_Index;

layout(location = 0) out vec3 v_WorldPosition;
//layout(location = 1) flat out vec3 v_WorldNormal;
layout(location = 1) out vec2 v_Uv;

layout(location = 2) flat out uint v_PerFaceIndex;
layout(location = 3) out float v_DistanceEdge;
layout(location = 4) out vec3 v_VertexPosition;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(set = 2, binding = 0) uniform Transform {
    mat4 Model;
};

void main() {
    vec4 world_position = Model * vec4(Vertex_Position, 1.0);
    v_WorldPosition = world_position.xyz;
    v_VertexPosition = Vertex_Position;
//    v_WorldNormal = mat3(Model) * Vertex_Normal;
    v_Uv = Vertex_Uv;
    v_PerFaceIndex = uint(Per_Face_Index);
    v_DistanceEdge = float(sign(Per_Face_Index));
    gl_Position = ViewProj * world_position;
}
