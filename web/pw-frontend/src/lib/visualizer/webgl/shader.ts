import type { Dictionary } from './util';

function error(msg: string) {
  console.error(msg);
}

const defaultShaderType = [
  "VERTEX_SHADER",
  "FRAGMENT_SHADER"
];

/// Create Shader from Source string
function loadShader(
  gl: WebGLRenderingContext,
  shaderSource: string,
  shaderType: number,
  opt_errorCallback: any,
): WebGLShader {
  var errFn = opt_errorCallback || error;
  // Create the shader object
  var shader = gl.createShader(shaderType);

  // Load the shader source
  gl.shaderSource(shader, shaderSource);

  // Compile the shader
  gl.compileShader(shader);

  // Check the compile status
  var compiled = gl.getShaderParameter(shader, gl.COMPILE_STATUS);
  if (!compiled) {
    // Something went wrong during compilation; get the error
    var lastError = gl.getShaderInfoLog(shader);
    errFn("*** Error compiling shader '" + shader + "':" + lastError);
    gl.deleteShader(shader);
    return null;
  }

  return shader;
}

/// Actually Create Program with Shader's
function createProgram(
  gl: WebGLRenderingContext,
  shaders: WebGLShader[],
  opt_attribs: string[],
  opt_locations: number[],
  opt_errorCallback: any,
): WebGLProgram {
  var errFn = opt_errorCallback || error;
  var program = gl.createProgram();
  shaders.forEach(function (shader) {
    gl.attachShader(program, shader);
  });
  if (opt_attribs) {
    opt_attribs.forEach(function (attrib, ndx) {
      gl.bindAttribLocation(
        program,
        opt_locations ? opt_locations[ndx] : ndx,
        attrib);
    });
  }
  gl.linkProgram(program);

  // Check the link status
  var linked = gl.getProgramParameter(program, gl.LINK_STATUS);
  if (!linked) {
    // something went wrong with the link
    var lastError = gl.getProgramInfoLog(program);
    errFn("Error in program linking:" + lastError);

    gl.deleteProgram(program);
    return null;
  }
  return program;
}

export class ShaderFactory {
  frag_source: string;
  vert_source: string;

  static async create_factory(frag_url: string, vert_url: string): Promise<ShaderFactory> {
    const sources = await Promise.all([
      fetch(frag_url).then((r) => r.text()),
      fetch(vert_url).then((r) => r.text()),
    ]);

    return new ShaderFactory(sources[0], sources[1]);
  }

  constructor(frag_source: string, vert_source: string ) {
    this.frag_source = frag_source;
    this.vert_source = vert_source;
  }

  create_shader(
    gl: WebGLRenderingContext,
    context?: Dictionary<string>,
    opt_attribs?: string[],
    opt_locations?: number[],
    opt_errorCallback?: any,
  ): Shader {
    let vert = this.vert_source.slice();
    let frag = this.frag_source.slice();
    for (let key in context) {
      vert = vert.replace(new RegExp("\\$" + key, 'g'), context[key]);
      frag = frag.replace(new RegExp("\\$" + key, 'g'), context[key]);
    }

    const shaders = [
      loadShader(gl, vert, gl.VERTEX_SHADER, opt_errorCallback),
      loadShader(gl, frag, gl.FRAGMENT_SHADER, opt_errorCallback),
    ];

    return new Shader(createProgram(gl, shaders, opt_attribs, opt_locations, opt_errorCallback));
  }
}

export class Shader {
  shader: WebGLProgram;
  uniformCache: Dictionary<WebGLUniformLocation>;
  attribCache: Dictionary<number>;

  static async createProgramFromUrls(
    gl: WebGLRenderingContext,
    vert_url: string,
    frag_url: string,
    context?: Dictionary<string>,
    opt_attribs?: string[],
    opt_locations?: number[],
    opt_errorCallback?: any,
  ): Promise<Shader> {
    const sources = (await Promise.all([
      fetch(vert_url).then((r) => r.text()),
      fetch(frag_url).then((r) => r.text()),
    ])).map(x => {
      for (let key in context) {
        x = x.replace(new RegExp("\\$" + key, 'g'), context[key]);
      }
      return x;
    });

    const shaders = [
      loadShader(gl, sources[0], 35633, opt_errorCallback),
      loadShader(gl, sources[1], 35632, opt_errorCallback),
    ];
    return new Shader(createProgram(gl, shaders, opt_attribs, opt_locations, opt_errorCallback));
  }

  constructor(shader: WebGLProgram) {
    this.shader = shader;
    this.uniformCache = {};
    this.attribCache = {};
  }

  bind(gl: WebGLRenderingContext) {
    gl.useProgram(this.shader);
  }

  // Different locations have different types :/
  getUniformLocation(gl: WebGLRenderingContext, name: string): WebGLUniformLocation {
    if (this.uniformCache[name] === undefined) {
      this.uniformCache[name] = gl.getUniformLocation(this.shader, name);
    }

    return this.uniformCache[name];
  }

  getAttribLocation(gl: WebGLRenderingContext, name: string): number {
    if (this.attribCache[name] === undefined) {
      this.attribCache[name] = gl.getAttribLocation(this.shader, name);
    }

    return this.attribCache[name];
  }

  uniform<T extends Uniform>(
    gl: WebGLRenderingContext,
    name: string,
    uniform: T,
  ) {
    this.bind(gl);
    const location = this.getUniformLocation(gl, name);
    if (location < 0) {
      console.error("No location found with name " + name);
    }

    uniform.setUniform(gl, location);
  }

  clear(gl: WebGLRenderingContext) {
    gl.deleteProgram(this.shader);
  }
}

export interface Uniform {
  setUniform(gl: WebGLRenderingContext, location: WebGLUniformLocation): void;
}

export class Uniform2fv implements Uniform {
  data: number[] | Float32Array;
  constructor(data: number[] | Float32Array) {
    this.data = data;
  }

  setUniform(gl: WebGLRenderingContext, location: WebGLUniformLocation) {
    gl.uniform2fv(location, this.data);
  }
}

export class Uniform3fv implements Uniform {
  data: number[] | Float32Array;
  constructor(data: number[] | Float32Array) {
    this.data = data;
  }

  setUniform(gl: WebGLRenderingContext, location: WebGLUniformLocation) {
    gl.uniform3fv(location, this.data);
  }
}

export class Uniform3f implements Uniform {
  x: number;
  y: number;
  z: number;

  constructor(x: number, y: number, z: number) {
    this.x = x;
    this.y = y;
    this.z = z;
  }

  setUniform(gl: WebGLRenderingContext, location: WebGLUniformLocation) {
    gl.uniform3f(location, this.x ,this.y, this.z);
  }
}

export class Uniform1iv implements Uniform {
  data: number[] | Int32List;
  constructor(data: number[] | Int32List) {
    this.data = data;
  }

  setUniform(gl: WebGLRenderingContext, location: WebGLUniformLocation) {
    gl.uniform1iv(location, this.data);
  }
}

export class Uniform1i implements Uniform {
  texture: number;

  constructor(texture: number) {
    this.texture = texture;
  }

  setUniform(gl: WebGLRenderingContext, location: WebGLUniformLocation) {
    gl.uniform1i(location, this.texture);
  }
}

export class Uniform1f implements Uniform {
  texture: number;

  constructor(texture: number) {
    this.texture = texture;
  }

  setUniform(gl: WebGLRenderingContext, location: WebGLUniformLocation) {
    gl.uniform1f(location, this.texture);
  }
}

export class Uniform2f implements Uniform {
  x: number;
  y: number;

  constructor(xy: number[]) {
    this.x = xy[0];
    this.y = xy[1];
  }

  setUniform(gl: WebGLRenderingContext, location: WebGLUniformLocation) {
    gl.uniform2f(location, this.x, this.y);
  }
}

export class Uniform4f implements Uniform {
  v0: number;
  v1: number;
  v2: number;
  v3: number;

  constructor(xyzw: number[]) {
    this.v0 = xyzw[0];
    this.v1 = xyzw[1];
    this.v2 = xyzw[2];
    this.v3 = xyzw[3];
  }

  setUniform(gl: WebGLRenderingContext, location: WebGLUniformLocation) {
    gl.uniform4f(location, this.v0, this.v1, this.v2, this.v3);
  }
}

export class UniformMatrix3fv implements Uniform {
  data: number[] | Float32Array;
  constructor(data: number[] | Float32Array) {
    this.data = data;
  }

  setUniform(gl: WebGLRenderingContext, location: WebGLUniformLocation) {
    gl.uniformMatrix3fv(location, false, this.data);
  }
}

export class UniformBool implements Uniform {
  data: boolean;
  constructor(data: boolean) {
    this.data = data;
  }

  setUniform(gl: WebGLRenderingContext, location: WebGLUniformLocation) {
    gl.uniform1i(location, this.data ? 1 : 0);
  }
}

export default Shader;