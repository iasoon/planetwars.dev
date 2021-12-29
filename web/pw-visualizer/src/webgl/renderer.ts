import type { IndexBuffer } from './buffer';
import type { VertexArray } from './vertexBufferLayout';
import type { Texture } from './texture';
import type { Dictionary } from './util';
import type { Shader, Uniform } from './shader';
import { Uniform1i } from './shader';

function sortedIndex(array, value) {
    var low = 0,
        high = array.length;

    while (low < high) {
        var mid = (low + high) >>> 1;
        if (array[mid] < value) low = mid + 1;
        else high = mid;
    }
    return low;
}

export interface Renderable {
    getUniforms() : Dictionary<Uniform>;
    render(gl: WebGLRenderingContext): void;
    updateVAOBuffer(gl: WebGLRenderingContext, index: number, data: number[]);
    updateIndexBuffer(gl: WebGLRenderingContext, data: number[]);
}

export class DefaultRenderable implements Renderable {
    ibo: IndexBuffer;
    va: VertexArray;
    shader: Shader;
    textures: Texture[];
    uniforms: Dictionary<Uniform>;

    constructor(
        ibo: IndexBuffer,
        va: VertexArray,
        shader: Shader,
        textures: Texture[],
        uniforms: Dictionary<Uniform>,
    ) {
        this.ibo = ibo;
        this.va = va;
        this.shader = shader;
        this.textures = textures;
        this.uniforms = uniforms;
    }

    getUniforms(): Dictionary<Uniform> {
        return this.uniforms;
    }

    updateVAOBuffer(gl: WebGLRenderingContext, index: number, data: number[]) {
        this.va.updateBuffer(gl, index, data);
    }

    updateIndexBuffer(gl: WebGLRenderingContext, data: number[]) {
        this.ibo.updateData(gl, data);
    }

    render(gl: WebGLRenderingContext): void {

        const indexBuffer = this.ibo;
        const vertexArray = this.va;
        const uniforms = this.uniforms;

        const shader = this.shader;
        const textures = this.textures;
        let texLocation = 0;

        for (let texture of textures) {

            shader.uniform(gl, texture.name, new Uniform1i(texLocation));
            texture.bind(gl, texLocation);

            texLocation ++;
            // if (texLocation > maxTextures) {
            //     console.error("Using too many textures, this is not supported yet\nUndefined behaviour!");
            // }
        }

        if (vertexArray && shader && uniforms) {
            for(let key in uniforms) {
                shader.uniform(gl, key, uniforms[key]);
            }

            vertexArray.bind(gl, shader);

            if (indexBuffer) {
                indexBuffer.bind(gl);
                gl.drawElements(gl.TRIANGLES, indexBuffer.getCount(), gl.UNSIGNED_SHORT, 0);
            } else {
                console.error("IndexBuffer is required to render, for now");
            }
        }

    }
}

export class Renderer {
    renderables: { [id: number] : [Renderable, boolean][]; };
    renderable_layers: number[];

    constructor() {
        this.renderables = {};
        this.renderable_layers = [];
    }

    updateUniform(i: number, f: (uniforms: Dictionary<Uniform>) => void, layer=0, ) {
        f(this.renderables[layer][i][0].getUniforms());
    }

    disableRenderable(i: number, layer=0) {
        this.renderables[layer][i][1] = false;
    }

    enableRenderable(i: number, layer=0) {
        this.renderables[layer][i][1] = true;
    }

    addRenderable(item: Renderable, layer=0): number {
        if(!this.renderables[layer]) {
            const idx = sortedIndex(this.renderable_layers, layer);
            this.renderable_layers.splice(idx, 0, layer);
            this.renderables[layer] = [];
        }

        this.renderables[layer].push([item, true]);
        return this.renderables[layer].length - 1;
    }

    addToDraw(indexBuffer: IndexBuffer, vertexArray: VertexArray, shader: Shader, uniforms?: Dictionary<Uniform>, texture?: Texture[], layer=0): number {
        return this.addRenderable(
            new DefaultRenderable(
                indexBuffer,
                vertexArray,
                shader,
                texture || [],
                uniforms || {},
            ), layer
        );
    }

    render(gl: WebGLRenderingContext, frameBuffer?: WebGLFramebuffer, width?: number, height?: number) {
        gl.bindFramebuffer(gl.FRAMEBUFFER, frameBuffer);
        gl.viewport(0, 0, width || gl.canvas.width, height || gl.canvas.height);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

        const maxTextures = gl.getParameter(gl.MAX_VERTEX_TEXTURE_IMAGE_UNITS);

        for (let layer of this.renderable_layers) {
            for (let [r, e] of this.renderables[layer]) {
                if (!e) continue;
                r.render(gl);
            }
        }
    }
}
