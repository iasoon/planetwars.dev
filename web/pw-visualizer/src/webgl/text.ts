import type { Dictionary } from "./util";
import type { Shader, UniformMatrix3fv } from "./shader";
import { Texture } from "./texture";
import { DefaultRenderable } from "./renderer";
import { IndexBuffer, VertexBuffer } from "./buffer";
import { VertexBufferLayout, VertexArray } from "./vertexBufferLayout";


export enum Align {
    Begin,
    End,
    Middle,
}

export class GlypInfo {
    x: number;
    y: number;
    width: number;
}

export class FontInfo {
    letterHeight: number;
    spaceWidth: number;
    spacing: number;
    textureWidth: number;
    textureHeight: number;
    glyphInfos: Dictionary<GlypInfo>;
}

export class LabelFactory {
    texture: Texture;
    font: FontInfo;
    shader: Shader;

    constructor(gl: WebGLRenderingContext, loc: string, font: FontInfo, shader: Shader) {
        this.texture = Texture.fromImage(gl, loc, 'font');
        this.font = font;
        this.shader = shader;
    }

    build(gl: WebGLRenderingContext, transform?: UniformMatrix3fv): Label {
        return new Label(gl, this.shader, this.texture, this.font, transform);
    }
}

export class Label {
    inner: DefaultRenderable;

    font: FontInfo;

    constructor(gl: WebGLRenderingContext, shader: Shader, tex: Texture, font: FontInfo, transform?: UniformMatrix3fv) {
        this.font = font;

        const uniforms = transform ? { "u_trans": transform, "u_trans_next": transform, } : {};
        const ib = new IndexBuffer(gl, []);
        const vb_pos = new VertexBuffer(gl, []);
        const vb_tex = new VertexBuffer(gl, []);

        const layout_pos = new VertexBufferLayout();
        layout_pos.push(gl.FLOAT, 2, 4, "a_position");

        const layout_tex = new VertexBufferLayout();
        layout_tex.push(gl.FLOAT, 2, 4, "a_texCoord");

        const vao = new VertexArray();
        vao.addBuffer(vb_pos, layout_pos);
        vao.addBuffer(vb_tex, layout_tex);

        this.inner = new DefaultRenderable(ib, vao, shader, [tex], uniforms);
    }

    getRenderable(): DefaultRenderable {
        return this.inner;
    }

    setText(gl: WebGLRenderingContext, text: string, h_align = Align.Begin, v_align = Align.Begin) {
        const idxs = [];
        const verts_pos = [];
        const verts_tex = [];

        const letterHeight = this.font.letterHeight / this.font.textureHeight;
        let xPos = 0;

        switch (h_align) {
            case Align.Begin:
                break;
            case Align.End:
                xPos = -1 * [...text].map(n => this.font.glyphInfos[n] ? this.font.glyphInfos[n].width : this.font.spaceWidth).reduce((a, b) => a + b, 0) / this.font.letterHeight;
                break;
            case Align.Middle:
                xPos = -1 * [...text].map(n => this.font.glyphInfos[n] ? this.font.glyphInfos[n].width : this.font.spaceWidth).reduce((a, b) => a + b, 0) / this.font.letterHeight / 2;
                break;
        }
        let yStart = 0;
        switch (v_align) {
            case Align.Begin:
                break;
            case Align.End:
                yStart = 1;
                break;
            case Align.Middle:
                yStart = 0.5;
                break;
        }

        let j = 0;
        for (let i = 0; i < text.length; i++) {
            const info = this.font.glyphInfos[text[i]];
            if (info) {
                const dx = info.width / this.font.letterHeight;
                const letterWidth = info.width / this.font.textureWidth;
                const x0 = info.x / this.font.textureWidth;
                const y0 = info.y / this.font.textureHeight;
                verts_pos.push(xPos,      yStart);
                verts_pos.push(xPos + dx, yStart);
                verts_pos.push(xPos,      yStart-1);
                verts_pos.push(xPos + dx, yStart-1);

                verts_tex.push(x0,               y0);
                verts_tex.push(x0 + letterWidth, y0);
                verts_tex.push(x0,               y0 + letterHeight);
                verts_tex.push(x0 + letterWidth, y0 + letterHeight);

                xPos += dx;

                idxs.push(j+0, j+1, j+2, j+1, j+2, j+3);
                j += 4;
            } else {
                // Just move xPos
                xPos += this.font.spaceWidth / this.font.letterHeight;
            }
        }

        this.inner.updateIndexBuffer(gl, idxs);
        this.inner.updateVAOBuffer(gl, 0, verts_pos);
        this.inner.updateVAOBuffer(gl, 1, verts_tex);
    }
}

export function defaultLabelFactory(gl: WebGLRenderingContext, shader: Shader): LabelFactory {
    const fontInfo = {
        letterHeight: 8,
        spaceWidth: 8,
        spacing: -1,
        textureWidth: 64,
        textureHeight: 40,
        glyphInfos: {
            'a': { x: 0, y: 0, width: 8, },
            'b': { x: 8, y: 0, width: 8, },
            'c': { x: 16, y: 0, width: 8, },
            'd': { x: 24, y: 0, width: 8, },
            'e': { x: 32, y: 0, width: 8, },
            'f': { x: 40, y: 0, width: 8, },
            'g': { x: 48, y: 0, width: 8, },
            'h': { x: 56, y: 0, width: 8, },
            'i': { x: 0, y: 8, width: 8, },
            'j': { x: 8, y: 8, width: 8, },
            'k': { x: 16, y: 8, width: 8, },
            'l': { x: 24, y: 8, width: 8, },
            'm': { x: 32, y: 8, width: 8, },
            'n': { x: 40, y: 8, width: 8, },
            'o': { x: 48, y: 8, width: 8, },
            'p': { x: 56, y: 8, width: 8, },
            'q': { x: 0, y: 16, width: 8, },
            'r': { x: 8, y: 16, width: 8, },
            's': { x: 16, y: 16, width: 8, },
            't': { x: 24, y: 16, width: 8, },
            'u': { x: 32, y: 16, width: 8, },
            'v': { x: 40, y: 16, width: 8, },
            'w': { x: 48, y: 16, width: 8, },
            'x': { x: 56, y: 16, width: 8, },
            'y': { x: 0, y: 24, width: 8, },
            'z': { x: 8, y: 24, width: 8, },
            '0': { x: 16, y: 24, width: 8, },
            '1': { x: 24, y: 24, width: 8, },
            '2': { x: 32, y: 24, width: 8, },
            '3': { x: 40, y: 24, width: 8, },
            '4': { x: 48, y: 24, width: 8, },
            '5': { x: 56, y: 24, width: 8, },
            '6': { x: 0, y: 32, width: 8, },
            '7': { x: 8, y: 32, width: 8, },
            '8': { x: 16, y: 32, width: 8, },
            '9': { x: 24, y: 32, width: 8, },
            '-': { x: 32, y: 32, width: 8, },
            '*': { x: 40, y: 32, width: 8, },
            '!': { x: 48, y: 32, width: 8, },
            '?': { x: 56, y: 32, width: 8, },
        },
    };

    return new LabelFactory(gl, '/static/res/assets/font.png', fontInfo, shader);
}
