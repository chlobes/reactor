varying highp vec4 col;
varying highp vec2 uv;
varying highp float blend;

uniform sampler2D tex;

void main() {
	gl_FragColor = (1.0 - blend) * col + blend * texture2D(tex,uv);
}
