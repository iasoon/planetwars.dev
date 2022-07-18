import { Uniform4f, Uniform1f, Uniform2f, ShaderFactory, UniformMatrix3fv, Uniform3f } from './shader';
import { resizeCanvasToDisplaySize, FPSCounter, onload2promise, Resizer, url_to_mesh } from "./util";
import { VertexBuffer, IndexBuffer } from './buffer';
import { VertexArray, VertexBufferLayout } from './vertexBufferLayout';
import { Renderer } from './renderer';
import { Texture } from './texture';
import * as assets from "../assets";

// const URL = window.location.origin+window.location.pathname;
// const LOCATION = URL.substring(0, URL.lastIndexOf("/") + 1);

async function create_texture_from_svg(gl: WebGLRenderingContext, name: string, path: string, width: number, height: number): Promise<Texture> {

    const [mesh, factory] = await Promise.all([
        url_to_mesh(path),
        ShaderFactory.create_factory(
            // assets.simpleFragmentShader,
            // assets.simpleVertexShader,
            // TODO: previously: this was the old code, which was not working.
            // what is the correct shader here?
            "static/shaders/frag/static_color.glsl", "static/shaders/vert/svg.glsl"
        )
    ]);

    const program = factory.create_shader(gl);
    const renderer = new Renderer();

    var positionBuffer = new VertexBuffer(gl, mesh.positions);
    var layout = new VertexBufferLayout();
    layout.push(gl.FLOAT, 3, 4, "a_position");

    const vao = new VertexArray();
    vao.addBuffer(positionBuffer, layout);

    program.bind(gl);
    vao.bind(gl, program);

    var indexBuffer = new IndexBuffer(gl, mesh.cells);
    indexBuffer.bind(gl);

    renderer.addToDraw(indexBuffer, vao, program, {});

    return Texture.fromRenderer(gl, name, width, height, renderer);
}


async function main() {

    // Get A WebGL context
    var canvas = <HTMLCanvasElement>document.getElementById("c");
    const resolution = [canvas.width, canvas.height];

    const resizer = new Resizer(canvas, [-10, -10, 20, 20], true);

    var gl = canvas.getContext("webgl");
    if (!gl) {
        return;
    }

    // TODO: do we still need this?
    const mesh = await url_to_mesh("static/res/images/earth.svg");
    const renderer = new Renderer();

    const factory = await ShaderFactory.create_factory(assets.simpleFragmentShader, assets.simpleVertexShader);
    const program = factory.create_shader(gl);

    var positionBuffer = new VertexBuffer(gl, mesh.positions);
    var layout = new VertexBufferLayout();
    layout.push(gl.FLOAT, 3, 4, "a_position");
    // layout.push(gl.FLOAT, 2, 4, "a_tex");

    const vao = new VertexArray();
    vao.addBuffer(positionBuffer, layout);

    resizeCanvasToDisplaySize(<HTMLCanvasElement>gl.canvas);

    // Tell WebGL how to convert from clip space to pixels
    gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);

    // Clear the canvas
    gl.clearColor(0, 0, 0, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

    program.bind(gl);
    vao.bind(gl, program);

    var indexBuffer = new IndexBuffer(gl, mesh.cells);
    indexBuffer.bind(gl);

    renderer.addToDraw(indexBuffer, vao, program, {});

    const counter = new FPSCounter();

    const step = function (time: number) {

        // console.log(resizer.get_viewbox());

        program.uniform(gl, "u_time", new Uniform1f(time * 0.001));
        program.uniform(gl, "u_mouse", new Uniform2f(resizer.get_mouse_pos()));
        program.uniform(gl, "u_viewbox", new Uniform4f(resizer.get_viewbox()));
        program.uniform(gl, "u_resolution", new Uniform2f(resolution));
        program.uniform(gl, "u_trans", new UniformMatrix3fv([1, 0, 0, 0, 1, 0, 0, 0, 1]));
        program.uniform(gl, "u_color", new Uniform3f(1.0, 0.5, 0.0));

        renderer.render(gl);

        counter.frame(time);
        requestAnimationFrame(step);
    }

    requestAnimationFrame(step);
}


main();

document.getElementById("loader").classList.remove("loading");

// const loader = document.getElementById("loader");
// setInterval(() => {
//     if (loader.classList.contains("loading")) {
//         loader.classList.remove("loading")
//     } else {
//         loader.classList.add("loading");
//     }
// }, 2000);
